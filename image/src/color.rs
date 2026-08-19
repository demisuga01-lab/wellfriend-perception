//! Color conversion and u8/f32 normalization helpers.

use wellfriend_perception_core::{ImageBuffer, PerceptionError, PerceptionResult, PixelFormat};

/// Converts between the MP2 baseline interleaved color layouts.
pub fn convert_color(input: &ImageBuffer, target: PixelFormat) -> PerceptionResult<ImageBuffer> {
    if input.pixel_format() == target {
        return input.view().to_owned();
    }
    let source = input.pixel_format();
    match (source, target) {
        (PixelFormat::Rgb8, PixelFormat::Bgr8) | (PixelFormat::Bgr8, PixelFormat::Rgb8) => {
            swap_red_blue(input, 3, false, target)
        }
        (PixelFormat::Rgba8, PixelFormat::Bgra8) | (PixelFormat::Bgra8, PixelFormat::Rgba8) => {
            swap_red_blue(input, 4, true, target)
        }
        (PixelFormat::Rgba8, PixelFormat::Rgb8) | (PixelFormat::Bgra8, PixelFormat::Bgr8) => {
            drop_alpha(input, target)
        }
        (PixelFormat::Rgb8, PixelFormat::Rgba8) | (PixelFormat::Bgr8, PixelFormat::Bgra8) => {
            add_alpha(input, target)
        }
        (PixelFormat::Rgb8, PixelFormat::Gray8) | (PixelFormat::Bgr8, PixelFormat::Gray8) => {
            rgb_like_to_gray(input)
        }
        (PixelFormat::Gray8, PixelFormat::Rgb8) => gray_to_rgb(input),
        _ => Err(PerceptionError::UnsupportedFormat {
            operation: "convert_color",
            format: format!("{source} -> {target}"),
        }),
    }
}

/// Converts RGB or BGR to Gray8 with BT.709 luminance coefficients: 0.2126, 0.7152, 0.0722.
pub fn grayscale(input: &ImageBuffer) -> PerceptionResult<ImageBuffer> {
    match input.pixel_format() {
        PixelFormat::Gray8 => input.view().to_owned(),
        PixelFormat::Rgb8 | PixelFormat::Bgr8 => rgb_like_to_gray(input),
        _ => Err(PerceptionError::UnsupportedFormat {
            operation: "grayscale",
            format: input.pixel_format().to_string(),
        }),
    }
}

/// Converts Gray8 to RGB8 by replicating the gray component into each color channel.
pub fn gray_to_rgb(input: &ImageBuffer) -> PerceptionResult<ImageBuffer> {
    if input.pixel_format() != PixelFormat::Gray8 {
        return Err(PerceptionError::UnsupportedFormat {
            operation: "gray_to_rgb",
            format: input.pixel_format().to_string(),
        });
    }
    let mut data = Vec::with_capacity(input.width() as usize * input.height() as usize * 3);
    for y in 0..input.height() {
        for gray in input.view().row(y)? {
            data.extend_from_slice(&[*gray, *gray, *gray]);
        }
    }
    ImageBuffer::new(input.width(), input.height(), PixelFormat::Rgb8, data)
}

/// Converts interleaved u8 samples to [0, 1] f32 samples.
pub fn u8_to_f32_normalized(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    if !input.pixel_format().is_u8_interleaved() {
        return Err(PerceptionError::UnsupportedFormat {
            operation: "u8_to_f32_normalized",
            format: input.pixel_format().to_string(),
        });
    }
    let channels =
        input
            .pixel_format()
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "u8_to_f32_normalized",
                format: input.pixel_format().to_string(),
            })?;
    let capacity = (input.width() as usize)
        .checked_mul(input.height() as usize)
        .and_then(|value| value.checked_mul(channels))
        .ok_or(PerceptionError::Overflow)?;
    let mut values = Vec::with_capacity(capacity);
    for y in 0..input.height() {
        values.extend(
            input
                .view()
                .row(y)?
                .iter()
                .map(|value| f32::from(*value) / 255.0),
        );
    }
    Ok(values)
}

/// Converts normalized f32 values into a packed Gray8 image after rejecting non-finite values.
pub fn normalized_f32_to_gray(
    width: u32,
    height: u32,
    values: &[f32],
) -> PerceptionResult<ImageBuffer> {
    let expected = width as usize * height as usize;
    if values.len() != expected {
        return Err(PerceptionError::InvalidBuffer {
            expected,
            actual: values.len(),
        });
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(PerceptionError::NumericFailure {
            reason: "normalized image contains non-finite value".into(),
        });
    }
    ImageBuffer::new(
        width,
        height,
        PixelFormat::Gray8,
        values
            .iter()
            .map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect(),
    )
}

fn swap_red_blue(
    input: &ImageBuffer,
    channels: usize,
    has_alpha: bool,
    target: PixelFormat,
) -> PerceptionResult<ImageBuffer> {
    let mut data = Vec::with_capacity(input.width() as usize * input.height() as usize * channels);
    for y in 0..input.height() {
        for pixel in input.view().row(y)?.chunks_exact(channels) {
            data.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
            if has_alpha {
                data.push(pixel[3]);
            }
        }
    }
    ImageBuffer::new(input.width(), input.height(), target, data)
}

fn drop_alpha(input: &ImageBuffer, target: PixelFormat) -> PerceptionResult<ImageBuffer> {
    let mut data = Vec::with_capacity(input.width() as usize * input.height() as usize * 3);
    for y in 0..input.height() {
        for pixel in input.view().row(y)?.chunks_exact(4) {
            data.extend_from_slice(&pixel[..3]);
        }
    }
    ImageBuffer::new(input.width(), input.height(), target, data)
}

fn add_alpha(input: &ImageBuffer, target: PixelFormat) -> PerceptionResult<ImageBuffer> {
    let mut data = Vec::with_capacity(input.width() as usize * input.height() as usize * 4);
    for y in 0..input.height() {
        for pixel in input.view().row(y)?.chunks_exact(3) {
            data.extend_from_slice(pixel);
            data.push(255);
        }
    }
    ImageBuffer::new(input.width(), input.height(), target, data)
}

fn rgb_like_to_gray(input: &ImageBuffer) -> PerceptionResult<ImageBuffer> {
    let is_bgr = input.pixel_format() == PixelFormat::Bgr8;
    let mut data = Vec::with_capacity(input.width() as usize * input.height() as usize);
    for y in 0..input.height() {
        for pixel in input.view().row(y)?.chunks_exact(3) {
            let (red, green, blue) = if is_bgr {
                (pixel[2], pixel[1], pixel[0])
            } else {
                (pixel[0], pixel[1], pixel[2])
            };
            data.push(
                (0.2126 * f32::from(red) + 0.7152 * f32::from(green) + 0.0722 * f32::from(blue))
                    .round() as u8,
            );
        }
    }
    ImageBuffer::new(input.width(), input.height(), PixelFormat::Gray8, data)
}
