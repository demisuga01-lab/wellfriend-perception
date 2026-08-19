//! Cropping, borders, normalization, and generic scalar convolution.

use wellfriend_perception_core::{
    ImageBuffer, PerceptionError, PerceptionResult, PixelFormat, RegionOfInterest,
};

/// Border behavior for scalar filters and padding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BorderMode {
    /// Constant component values, one for every image channel.
    Constant(Vec<u8>),
    /// Replicate nearest valid edge pixel.
    Replicate,
    /// Reflect coordinates around image edges without repeating endpoints.
    Reflect,
}

/// Odd-sized floating-point convolution kernel.
#[derive(Clone, Debug, PartialEq)]
pub struct Kernel {
    /// Kernel width.
    pub width: u32,
    /// Kernel height.
    pub height: u32,
    /// Row-major values.
    pub values: Vec<f32>,
}

impl Kernel {
    /// Validates an odd non-empty kernel shape and finite coefficients.
    pub fn new(width: u32, height: u32, values: Vec<f32>) -> PerceptionResult<Self> {
        if width == 0 || height == 0 || width % 2 == 0 || height % 2 == 0 {
            return Err(PerceptionError::InvalidDimensions { width, height });
        }
        let expected = width as usize * height as usize;
        if values.len() != expected {
            return Err(PerceptionError::InvalidBuffer {
                expected,
                actual: values.len(),
            });
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(PerceptionError::NumericFailure {
                reason: "kernel contains non-finite coefficient".into(),
            });
        }
        Ok(Self {
            width,
            height,
            values,
        })
    }

    /// Returns a checked coefficient.
    pub fn at(&self, x: u32, y: u32) -> PerceptionResult<f32> {
        if x >= self.width || y >= self.height {
            return Err(PerceptionError::OutOfBounds {
                reason: format!(
                    "kernel coefficient {x},{y} exceeds {}x{}",
                    self.width, self.height
                ),
            });
        }
        self.values
            .get(y as usize * self.width as usize + x as usize)
            .copied()
            .ok_or(PerceptionError::InvalidBuffer {
                expected: self.width as usize * self.height as usize,
                actual: self.values.len(),
            })
    }
}

/// Standard baseline kernels.
pub mod kernels {
    use super::Kernel;
    use wellfriend_perception_core::PerceptionResult;

    /// 3x3 normalized box blur.
    pub fn box3() -> PerceptionResult<Kernel> {
        Kernel::new(3, 3, vec![1.0 / 9.0; 9])
    }
    /// 3x3 Gaussian blur with sigma approximately 0.85.
    pub fn gaussian3() -> PerceptionResult<Kernel> {
        Kernel::new(
            3,
            3,
            vec![1.0, 2.0, 1.0, 2.0, 4.0, 2.0, 1.0, 2.0, 1.0]
                .into_iter()
                .map(|value| value / 16.0)
                .collect(),
        )
    }
    /// 5x5 binomial Gaussian blur.
    pub fn gaussian5() -> PerceptionResult<Kernel> {
        let row = [1.0, 4.0, 6.0, 4.0, 1.0];
        let values = row
            .into_iter()
            .flat_map(|a| row.into_iter().map(move |b| a * b / 256.0))
            .collect();
        Kernel::new(5, 5, values)
    }
    /// Sobel horizontal derivative.
    pub fn sobel_x() -> PerceptionResult<Kernel> {
        Kernel::new(3, 3, vec![-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0])
    }
    /// Sobel vertical derivative.
    pub fn sobel_y() -> PerceptionResult<Kernel> {
        Kernel::new(3, 3, vec![-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0])
    }
    /// Scharr horizontal derivative.
    pub fn scharr_x() -> PerceptionResult<Kernel> {
        Kernel::new(3, 3, vec![-3.0, 0.0, 3.0, -10.0, 0.0, 10.0, -3.0, 0.0, 3.0])
    }
    /// Scharr vertical derivative.
    pub fn scharr_y() -> PerceptionResult<Kernel> {
        Kernel::new(3, 3, vec![-3.0, -10.0, -3.0, 0.0, 0.0, 0.0, 3.0, 10.0, 3.0])
    }
    /// Four-neighbor Laplacian.
    pub fn laplacian() -> PerceptionResult<Kernel> {
        Kernel::new(3, 3, vec![0.0, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0])
    }
    /// 3x3 sharpen kernel.
    pub fn sharpen() -> PerceptionResult<Kernel> {
        Kernel::new(3, 3, vec![0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0])
    }
}

/// Copies a checked region of interest into a packed owned image.
pub fn crop(input: &ImageBuffer, roi: RegionOfInterest) -> PerceptionResult<ImageBuffer> {
    input.roi(roi)?.to_owned()
}

/// Compatibility helper that validates an x/y/width/height crop.
pub fn crop_xywh(
    input: &ImageBuffer,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> PerceptionResult<ImageBuffer> {
    crop(
        input,
        RegionOfInterest::within(input.shape(), x, y, width, height)?,
    )
}

/// Pads all sides with the selected border behavior.
pub fn pad(
    input: &ImageBuffer,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    border: BorderMode,
) -> PerceptionResult<ImageBuffer> {
    require_u8(input, "pad")?;
    let width = input
        .width()
        .checked_add(left)
        .and_then(|value| value.checked_add(right))
        .ok_or(PerceptionError::Overflow)?;
    let height = input
        .height()
        .checked_add(top)
        .and_then(|value| value.checked_add(bottom))
        .ok_or(PerceptionError::Overflow)?;
    let channels =
        input
            .pixel_format()
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "pad",
                format: input.pixel_format().to_string(),
            })?;
    if let BorderMode::Constant(values) = &border {
        if values.len() != channels {
            return Err(PerceptionError::InvalidBuffer {
                expected: channels,
                actual: values.len(),
            });
        }
    }
    let mut data = vec![0; width as usize * height as usize * channels];
    for y in 0..height {
        for x in 0..width {
            let source_x = x as i32 - left as i32;
            let source_y = y as i32 - top as i32;
            for channel in 0..channels {
                data[(y as usize * width as usize + x as usize) * channels + channel] =
                    border_sample(input, source_x, source_y, channel, &border)?;
            }
        }
    }
    ImageBuffer::new(width, height, input.pixel_format(), data)
}

/// Constant-value padding convenience helper; the supplied value is repeated in every channel.
pub fn pad_constant(
    input: &ImageBuffer,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    value: u8,
) -> PerceptionResult<ImageBuffer> {
    let channels =
        input
            .pixel_format()
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "pad_constant",
                format: input.pixel_format().to_string(),
            })?;
    pad(
        input,
        left,
        top,
        right,
        bottom,
        BorderMode::Constant(vec![value; channels]),
    )
}

/// Scales Gray8 samples to the unit interval without allocating a typed f32 image buffer.
pub fn scale_to_unit_gray(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    require_gray(input, "scale_to_unit_gray")?;
    Ok(packed_u8(input)?
        .iter()
        .map(|value| f32::from(*value) / 255.0)
        .collect())
}

/// Min/max normalizes Gray8 values into the full eight-bit range.
pub fn min_max_normalize_gray(input: &ImageBuffer) -> PerceptionResult<ImageBuffer> {
    require_gray(input, "min_max_normalize_gray")?;
    let packed = packed_u8(input)?;
    let min = *packed.iter().min().ok_or(PerceptionError::InvalidBuffer {
        expected: 1,
        actual: 0,
    })?;
    let max = *packed.iter().max().ok_or(PerceptionError::InvalidBuffer {
        expected: 1,
        actual: 0,
    })?;
    if min == max {
        return input.view().to_owned();
    }
    ImageBuffer::new(
        input.width(),
        input.height(),
        PixelFormat::Gray8,
        packed
            .iter()
            .map(|value| ((u16::from(*value - min) * 255) / u16::from(max - min)) as u8)
            .collect(),
    )
}

/// Compatibility alias for min/max normalization.
pub fn normalize_gray(input: &ImageBuffer) -> PerceptionResult<ImageBuffer> {
    min_max_normalize_gray(input)
}

/// Applies `(value - mean) / standard_deviation` to Gray8 samples.
pub fn mean_std_normalize_gray(
    input: &ImageBuffer,
    mean: f32,
    standard_deviation: f32,
) -> PerceptionResult<Vec<f32>> {
    require_gray(input, "mean_std_normalize_gray")?;
    if !mean.is_finite() || !standard_deviation.is_finite() || standard_deviation <= 0.0 {
        return Err(PerceptionError::NumericFailure {
            reason: "mean/std normalization requires finite mean and positive standard deviation"
                .into(),
        });
    }
    Ok(packed_u8(input)?
        .iter()
        .map(|value| (f32::from(*value) - mean) / standard_deviation)
        .collect())
}

/// Clamps Gray8 values in place into an inclusive range.
pub fn clamp_gray(input: &ImageBuffer, minimum: u8, maximum: u8) -> PerceptionResult<ImageBuffer> {
    require_gray(input, "clamp_gray")?;
    if minimum > maximum {
        return Err(PerceptionError::NumericFailure {
            reason: "clamp minimum exceeds maximum".into(),
        });
    }
    ImageBuffer::new(
        input.width(),
        input.height(),
        PixelFormat::Gray8,
        packed_u8(input)?
            .iter()
            .map(|value| (*value).clamp(minimum, maximum))
            .collect(),
    )
}

/// Applies a positive gamma curve to Gray8 values.
pub fn gamma_gray(input: &ImageBuffer, gamma: f32) -> PerceptionResult<ImageBuffer> {
    require_gray(input, "gamma_gray")?;
    if !gamma.is_finite() || gamma <= 0.0 {
        return Err(PerceptionError::NumericFailure {
            reason: "gamma must be finite and positive".into(),
        });
    }
    ImageBuffer::new(
        input.width(),
        input.height(),
        PixelFormat::Gray8,
        packed_u8(input)?
            .iter()
            .map(|value| ((f32::from(*value) / 255.0).powf(gamma) * 255.0).round() as u8)
            .collect(),
    )
}

/// Applies a generic odd-sized Gray8 convolution and returns signed f32 samples.
pub fn convolve_gray(
    input: &ImageBuffer,
    kernel: &Kernel,
    border: &BorderMode,
) -> PerceptionResult<Vec<f32>> {
    require_gray(input, "convolve_gray")?;
    let radius_x = (kernel.width / 2) as i32;
    let radius_y = (kernel.height / 2) as i32;
    let mut output = vec![0.0; input.width() as usize * input.height() as usize];
    for y in 0..input.height() {
        for x in 0..input.width() {
            let mut total = 0.0;
            for ky in 0..kernel.height {
                for kx in 0..kernel.width {
                    total += f32::from(border_sample(
                        input,
                        x as i32 + kx as i32 - radius_x,
                        y as i32 + ky as i32 - radius_y,
                        0,
                        border,
                    )?) * kernel.at(kx, ky)?;
                }
            }
            output[y as usize * input.width() as usize + x as usize] = total;
        }
    }
    Ok(output)
}

/// Applies a separable Gray8 convolution using horizontal then vertical kernel vectors.
pub fn separable_convolve_gray(
    input: &ImageBuffer,
    horizontal: &[f32],
    vertical: &[f32],
    border: &BorderMode,
) -> PerceptionResult<Vec<f32>> {
    require_gray(input, "separable_convolve_gray")?;
    if horizontal.is_empty()
        || vertical.is_empty()
        || horizontal.len() % 2 == 0
        || vertical.len() % 2 == 0
        || horizontal
            .iter()
            .chain(vertical)
            .any(|value| !value.is_finite())
    {
        return Err(PerceptionError::NumericFailure {
            reason: "separable kernels must be finite, non-empty, and odd sized".into(),
        });
    }
    let width = input.width();
    let height = input.height();
    let mut temporary = vec![0.0; width as usize * height as usize];
    let horizontal_radius = (horizontal.len() / 2) as i32;
    for y in 0..height {
        for x in 0..width {
            temporary[y as usize * width as usize + x as usize] = horizontal
                .iter()
                .enumerate()
                .map(|(index, weight)| {
                    Ok(f32::from(border_sample(
                        input,
                        x as i32 + index as i32 - horizontal_radius,
                        y as i32,
                        0,
                        border,
                    )?) * *weight)
                })
                .collect::<PerceptionResult<Vec<_>>>()?
                .into_iter()
                .sum();
        }
    }
    let vertical_radius = (vertical.len() / 2) as i32;
    let mut output = vec![0.0; temporary.len()];
    for y in 0..height {
        for x in 0..width {
            let mut total = 0.0;
            for (index, weight) in vertical.iter().enumerate() {
                let sy = y as i32 + index as i32 - vertical_radius;
                let sample = if sy < 0 || sy >= height as i32 {
                    match border {
                        BorderMode::Constant(values) => f32::from(values[0]),
                        BorderMode::Replicate => {
                            temporary[sy.clamp(0, height as i32 - 1) as usize * width as usize
                                + x as usize]
                        }
                        BorderMode::Reflect => {
                            temporary
                                [reflect(sy, height as i32) as usize * width as usize + x as usize]
                        }
                    }
                } else {
                    temporary[sy as usize * width as usize + x as usize]
                };
                total += sample * *weight;
            }
            output[y as usize * width as usize + x as usize] = total;
        }
    }
    Ok(output)
}

/// Compatibility helper for an explicit 3x3 zero-padded convolution.
pub fn convolve_gray_3x3(
    input: &ImageBuffer,
    coefficients: [f32; 9],
) -> PerceptionResult<Vec<f32>> {
    convolve_gray(
        input,
        &Kernel::new(3, 3, coefficients.to_vec())?,
        &BorderMode::Constant(vec![0]),
    )
}

fn require_gray(input: &ImageBuffer, operation: &'static str) -> PerceptionResult<()> {
    if input.pixel_format() != PixelFormat::Gray8 {
        return Err(PerceptionError::UnsupportedFormat {
            operation,
            format: input.pixel_format().to_string(),
        });
    }
    Ok(())
}
fn require_u8(input: &ImageBuffer, operation: &'static str) -> PerceptionResult<()> {
    if !input.pixel_format().is_u8_interleaved() {
        return Err(PerceptionError::UnsupportedFormat {
            operation,
            format: input.pixel_format().to_string(),
        });
    }
    Ok(())
}

pub(crate) fn border_sample(
    input: &ImageBuffer,
    x: i32,
    y: i32,
    channel: usize,
    border: &BorderMode,
) -> PerceptionResult<u8> {
    if x >= 0 && y >= 0 && x < input.width() as i32 && y < input.height() as i32 {
        return input.get_u8(x as u32, y as u32, channel);
    }
    match border {
        BorderMode::Constant(values) => {
            values
                .get(channel)
                .copied()
                .ok_or(PerceptionError::InvalidBuffer {
                    expected: input.pixel_format().channel_count().unwrap_or_default(),
                    actual: values.len(),
                })
        }
        BorderMode::Replicate => input.get_u8(
            x.clamp(0, input.width() as i32 - 1) as u32,
            y.clamp(0, input.height() as i32 - 1) as u32,
            channel,
        ),
        BorderMode::Reflect => input.get_u8(
            reflect(x, input.width() as i32) as u32,
            reflect(y, input.height() as i32) as u32,
            channel,
        ),
    }
}

/// Copies only logical image rows, excluding any per-row stride padding.
pub(crate) fn packed_u8(input: &ImageBuffer) -> PerceptionResult<Vec<u8>> {
    let channels =
        input
            .pixel_format()
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "packed_u8",
                format: input.pixel_format().to_string(),
            })?;
    let row_length = (input.width() as usize)
        .checked_mul(channels)
        .ok_or(PerceptionError::Overflow)?;
    let capacity = row_length
        .checked_mul(input.height() as usize)
        .ok_or(PerceptionError::Overflow)?;
    let view = input.view();
    let mut output = Vec::with_capacity(capacity);
    for y in 0..input.height() {
        output.extend_from_slice(view.row(y)?);
    }
    Ok(output)
}
pub(crate) fn reflect(index: i32, length: i32) -> i32 {
    if length <= 1 {
        return 0;
    }
    let period = 2 * length - 2;
    let value = index.rem_euclid(period);
    if value >= length {
        period - value
    } else {
        value
    }
}
