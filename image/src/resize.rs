//! Nearest and bilinear resampling for interleaved u8 images.

use wellfriend_perception_core::{
    ImageBuffer, PerceptionError, PerceptionResult,
    geometry::{SamplingLayout, sample_bilinear},
};

/// Resamples using nearest-neighbor lookup.
pub fn resize_nearest(
    input: &ImageBuffer,
    width: u32,
    height: u32,
) -> PerceptionResult<ImageBuffer> {
    validate_resize(input, width, height, "resize_nearest")?;
    let channels =
        input
            .pixel_format()
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "resize_nearest",
                format: input.pixel_format().to_string(),
            })?;
    let mut data = vec![0; width as usize * height as usize * channels];
    for y in 0..height {
        let source_y = ((y as f32 + 0.5) * input.height() as f32 / height as f32 - 0.5)
            .round()
            .clamp(0.0, (input.height() - 1) as f32) as u32;
        for x in 0..width {
            let source_x = ((x as f32 + 0.5) * input.width() as f32 / width as f32 - 0.5)
                .round()
                .clamp(0.0, (input.width() - 1) as f32) as u32;
            for channel in 0..channels {
                data[(y as usize * width as usize + x as usize) * channels + channel] =
                    input.get_u8(source_x, source_y, channel)?;
            }
        }
    }
    ImageBuffer::new(width, height, input.pixel_format(), data)
}

/// Resamples using bilinear interpolation, sampling at pixel centers and replicating edges.
pub fn resize_bilinear(
    input: &ImageBuffer,
    width: u32,
    height: u32,
) -> PerceptionResult<ImageBuffer> {
    validate_resize(input, width, height, "resize_bilinear")?;
    let channels =
        input
            .pixel_format()
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "resize_bilinear",
                format: input.pixel_format().to_string(),
            })?;
    let mut data = vec![0; width as usize * height as usize * channels];
    let layout = SamplingLayout {
        width: input.width(),
        height: input.height(),
        stride: input.stride().0,
        channels,
    };
    for y in 0..height {
        let source_y = ((y as f32 + 0.5) * input.height() as f32 / height as f32 - 0.5)
            .clamp(0.0, (input.height() - 1) as f32);
        for x in 0..width {
            let source_x = ((x as f32 + 0.5) * input.width() as f32 / width as f32 - 0.5)
                .clamp(0.0, (input.width() - 1) as f32);
            for channel in 0..channels {
                data[(y as usize * width as usize + x as usize) * channels + channel] =
                    sample_bilinear(input.as_bytes(), layout, source_x, source_y, channel, 0)?;
            }
        }
    }
    ImageBuffer::new(width, height, input.pixel_format(), data)
}

fn validate_resize(
    input: &ImageBuffer,
    width: u32,
    height: u32,
    operation: &'static str,
) -> PerceptionResult<()> {
    if width == 0 || height == 0 {
        return Err(PerceptionError::InvalidDimensions { width, height });
    }
    if !input.pixel_format().is_u8_interleaved() {
        return Err(PerceptionError::UnsupportedFormat {
            operation,
            format: input.pixel_format().to_string(),
        });
    }
    Ok(())
}
