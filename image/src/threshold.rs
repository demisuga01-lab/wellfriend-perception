//! Fixed, Otsu, and adaptive-mean thresholding baselines.

use crate::{BorderMode, border_sample, histogram_gray, packed_u8};
use wellfriend_perception_core::{ImageBuffer, PerceptionError, PerceptionResult, PixelFormat};

/// Applies an inclusive fixed threshold to Gray8 input.
pub fn threshold_gray(input: &ImageBuffer, threshold: u8) -> PerceptionResult<ImageBuffer> {
    require_gray(input, "threshold_gray")?;
    ImageBuffer::new(
        input.width(),
        input.height(),
        PixelFormat::Gray8,
        packed_u8(input)?
            .iter()
            .map(|value| if *value >= threshold { 255 } else { 0 })
            .collect(),
    )
}

/// Computes the Otsu threshold that maximizes inter-class variance.
pub fn otsu_threshold(input: &ImageBuffer) -> PerceptionResult<u8> {
    let histogram = histogram_gray(input)?;
    let total: f32 = histogram.bins.iter().map(|value| *value as f32).sum();
    if total == 0.0 {
        return Err(PerceptionError::InvalidBuffer {
            expected: 1,
            actual: 0,
        });
    }
    let sum_total: f32 = histogram
        .bins
        .iter()
        .enumerate()
        .map(|(index, count)| index as f32 * *count as f32)
        .sum();
    let mut weight_background = 0.0;
    let mut sum_background = 0.0;
    let mut best_variance = -1.0;
    let mut best = 0;
    for threshold in 0..256 {
        weight_background += histogram.bins[threshold] as f32;
        if weight_background == 0.0 {
            continue;
        }
        let weight_foreground = total - weight_background;
        if weight_foreground == 0.0 {
            break;
        }
        sum_background += threshold as f32 * histogram.bins[threshold] as f32;
        let mean_background = sum_background / weight_background;
        let mean_foreground = (sum_total - sum_background) / weight_foreground;
        let variance =
            weight_background * weight_foreground * (mean_background - mean_foreground).powi(2);
        if variance > best_variance {
            best_variance = variance;
            best = threshold as u8;
        }
    }
    Ok(best)
}

/// Applies a local mean threshold with a positive odd square window.
pub fn adaptive_mean_threshold(
    input: &ImageBuffer,
    window: u32,
    offset: f32,
) -> PerceptionResult<ImageBuffer> {
    require_gray(input, "adaptive_mean_threshold")?;
    if window == 0 || window % 2 == 0 || !offset.is_finite() {
        return Err(PerceptionError::InvalidDimensions {
            width: window,
            height: window,
        });
    }
    let radius = (window / 2) as i32;
    let mut data = vec![0; input.width() as usize * input.height() as usize];
    for y in 0..input.height() {
        for x in 0..input.width() {
            let mut sum = 0u32;
            for oy in -radius..=radius {
                for ox in -radius..=radius {
                    sum += u32::from(border_sample(
                        input,
                        x as i32 + ox,
                        y as i32 + oy,
                        0,
                        &BorderMode::Replicate,
                    )?);
                }
            }
            let mean = sum as f32 / (window * window) as f32;
            data[y as usize * input.width() as usize + x as usize] =
                if f32::from(input.get_u8(x, y, 0)?) >= mean - offset {
                    255
                } else {
                    0
                };
        }
    }
    ImageBuffer::new(input.width(), input.height(), PixelFormat::Gray8, data)
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
