//! Baseline scalar filters and derivative products.

use crate::{BorderMode, border_sample, convolve_gray, kernels, packed_u8};
use wellfriend_perception_core::{ImageBuffer, PerceptionError, PerceptionResult, PixelFormat};

/// Applies a 3x3 box blur.
pub fn box_blur_gray(input: &ImageBuffer) -> PerceptionResult<ImageBuffer> {
    to_gray_image(
        input,
        convolve_gray(input, &kernels::box3()?, &BorderMode::Replicate)?,
    )
}
/// Applies a 3x3 Gaussian blur.
pub fn gaussian_blur_gray(input: &ImageBuffer) -> PerceptionResult<ImageBuffer> {
    to_gray_image(
        input,
        convolve_gray(input, &kernels::gaussian3()?, &BorderMode::Replicate)?,
    )
}
/// Applies a 3x3 median filter with replicated borders.
pub fn median_blur_3x3(input: &ImageBuffer) -> PerceptionResult<ImageBuffer> {
    require_gray(input, "median_blur_3x3")?;
    let mut data = vec![0; input.width() as usize * input.height() as usize];
    for y in 0..input.height() {
        for x in 0..input.width() {
            let mut values = [0u8; 9];
            let mut index = 0;
            for oy in -1..=1 {
                for ox in -1..=1 {
                    values[index] = border_sample(
                        input,
                        x as i32 + ox,
                        y as i32 + oy,
                        0,
                        &BorderMode::Replicate,
                    )?;
                    index += 1;
                }
            }
            values.sort_unstable();
            data[y as usize * input.width() as usize + x as usize] = values[4];
        }
    }
    ImageBuffer::new(input.width(), input.height(), PixelFormat::Gray8, data)
}
/// Adds a scaled high-frequency residual to the source image.
pub fn unsharp_mask_gray(input: &ImageBuffer, amount: f32) -> PerceptionResult<ImageBuffer> {
    require_gray(input, "unsharp_mask_gray")?;
    if !amount.is_finite() || amount < 0.0 {
        return Err(PerceptionError::NumericFailure {
            reason: "unsharp amount must be finite and non-negative".into(),
        });
    }
    let blurred = gaussian_blur_gray(input)?;
    ImageBuffer::new(
        input.width(),
        input.height(),
        PixelFormat::Gray8,
        packed_u8(input)?
            .iter()
            .zip(packed_u8(&blurred)?)
            .map(|(source, smooth)| {
                (f32::from(*source) + amount * (f32::from(*source) - f32::from(smooth)))
                    .round()
                    .clamp(0.0, 255.0) as u8
            })
            .collect(),
    )
}
/// Sobel horizontal response.
pub fn gradient_x(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    convolve_gray(input, &kernels::sobel_x()?, &BorderMode::Replicate)
}
/// Sobel vertical response.
pub fn gradient_y(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    convolve_gray(input, &kernels::sobel_y()?, &BorderMode::Replicate)
}
/// Gradient magnitude from Sobel X and Y.
pub fn gradient_magnitude(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    let x = gradient_x(input)?;
    let y = gradient_y(input)?;
    Ok(x.into_iter().zip(y).map(|(x, y)| x.hypot(y)).collect())
}
/// Gradient orientation in radians from Sobel X and Y.
pub fn gradient_orientation(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    let x = gradient_x(input)?;
    let y = gradient_y(input)?;
    Ok(x.into_iter().zip(y).map(|(x, y)| y.atan2(x)).collect())
}
/// Scharr horizontal response.
pub fn scharr_x(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    convolve_gray(input, &kernels::scharr_x()?, &BorderMode::Replicate)
}
/// Scharr vertical response.
pub fn scharr_y(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    convolve_gray(input, &kernels::scharr_y()?, &BorderMode::Replicate)
}
/// Laplacian response.
pub fn laplacian(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    convolve_gray(input, &kernels::laplacian()?, &BorderMode::Replicate)
}
/// Compatibility Sobel magnitude helper.
pub fn sobel_magnitude_gray(input: &ImageBuffer) -> PerceptionResult<Vec<f32>> {
    gradient_magnitude(input)
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
fn to_gray_image(input: &ImageBuffer, values: Vec<f32>) -> PerceptionResult<ImageBuffer> {
    require_gray(input, "to_gray_image")?;
    ImageBuffer::new(
        input.width(),
        input.height(),
        PixelFormat::Gray8,
        values
            .into_iter()
            .map(|value| value.round().clamp(0.0, 255.0) as u8)
            .collect(),
    )
}
