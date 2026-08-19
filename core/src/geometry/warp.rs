//! Domain-neutral projective warping and dense remapping for interleaved u8 images.

use crate::{
    DenseWarpField, ImageBuffer, ImageShape, PerceptionError, PerceptionResult, Point2, Transform2D,
};

use super::{SamplingLayout, sample_bilinear, sample_nearest};

/// Sampling method for image warps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplingMode {
    /// Select the closest source sample.
    Nearest,
    /// Blend four neighboring source samples.
    Bilinear,
}

/// Out-of-bounds behavior for baseline generic warps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarpBorder {
    /// Fill outside samples with this component value.
    Constant(u8),
    /// Clamp coordinates to the nearest source edge.
    Replicate,
}

/// Applies a projective transform using inverse mapping from each output pixel.
pub fn warp_perspective(
    input: &ImageBuffer,
    transform: Transform2D,
    output: ImageShape,
    sampling: SamplingMode,
    border: WarpBorder,
) -> PerceptionResult<ImageBuffer> {
    if !input.pixel_format().is_u8_interleaved() {
        return Err(PerceptionError::UnsupportedFormat {
            operation: "warp_perspective",
            format: input.pixel_format().to_string(),
        });
    }
    let inverse = transform.inverse()?;
    let channels =
        input
            .pixel_format()
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "warp_perspective",
                format: input.pixel_format().to_string(),
            })?;
    let mut data = vec![0; output.width as usize * output.height as usize * channels];
    for y in 0..output.height {
        for x in 0..output.width {
            let source = inverse.apply_point(Point2::new(x as f32, y as f32))?;
            for channel in 0..channels {
                let value = sample(input, source, channel, sampling, border)?;
                data[(y as usize * output.width as usize + x as usize) * channels + channel] =
                    value;
            }
        }
    }
    ImageBuffer::new(output.width, output.height, input.pixel_format(), data)
}

/// Applies a dense output-to-source coordinate field using the same scalar sampling model.
pub fn remap(
    input: &ImageBuffer,
    field: &DenseWarpField,
    sampling: SamplingMode,
    border: WarpBorder,
) -> PerceptionResult<ImageBuffer> {
    if !input.pixel_format().is_u8_interleaved() {
        return Err(PerceptionError::UnsupportedFormat {
            operation: "remap",
            format: input.pixel_format().to_string(),
        });
    }
    let expected = field.width as usize * field.height as usize;
    if field.vectors.len() != expected {
        return Err(PerceptionError::InvalidBuffer {
            expected,
            actual: field.vectors.len(),
        });
    }
    let channels =
        input
            .pixel_format()
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "remap",
                format: input.pixel_format().to_string(),
            })?;
    let mut data = vec![0; expected * channels];
    for (index, source) in field.vectors.iter().enumerate() {
        for channel in 0..channels {
            data[index * channels + channel] = sample(input, *source, channel, sampling, border)?;
        }
    }
    ImageBuffer::new(field.width, field.height, input.pixel_format(), data)
}

fn sample(
    input: &ImageBuffer,
    point: Point2,
    channel: usize,
    sampling: SamplingMode,
    border: WarpBorder,
) -> PerceptionResult<u8> {
    let width = input.width();
    let height = input.height();
    let channels =
        input
            .pixel_format()
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "warp sample",
                format: input.pixel_format().to_string(),
            })?;
    let stride = input.stride().0;
    let bytes = input.as_bytes();
    let layout = SamplingLayout {
        width,
        height,
        stride,
        channels,
    };
    let outside = point.x < 0.0
        || point.y < 0.0
        || point.x > (width - 1) as f32
        || point.y > (height - 1) as f32;
    let (x, y, fallback) = match border {
        WarpBorder::Constant(value) => (point.x, point.y, value),
        WarpBorder::Replicate => (
            point.x.clamp(0.0, (width - 1) as f32),
            point.y.clamp(0.0, (height - 1) as f32),
            0,
        ),
    };
    if outside
        && matches!(border, WarpBorder::Constant(_))
        && matches!(sampling, SamplingMode::Nearest)
    {
        return Ok(fallback);
    }
    match sampling {
        SamplingMode::Nearest => {
            Ok(sample_nearest(bytes, layout, x, y, channel).unwrap_or(fallback))
        }
        SamplingMode::Bilinear => sample_bilinear(bytes, layout, x, y, channel, fallback),
    }
}
