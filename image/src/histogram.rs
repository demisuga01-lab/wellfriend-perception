//! Histograms, cumulative counts, and percentile estimation.

use wellfriend_perception_core::{ImageBuffer, PerceptionError, PerceptionResult, PixelFormat};

/// Fixed 256-bin count histogram.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Histogram {
    /** Bins indexed by sample value. */
    pub bins: [u32; 256],
}

/// Computes a Gray8 histogram.
pub fn histogram_gray(input: &ImageBuffer) -> PerceptionResult<Histogram> {
    if input.pixel_format() != PixelFormat::Gray8 {
        return Err(PerceptionError::UnsupportedFormat {
            operation: "histogram_gray",
            format: input.pixel_format().to_string(),
        });
    }
    let mut bins = [0; 256];
    let view = input.view();
    for y in 0..input.height() {
        for value in view.row(y)? {
            bins[*value as usize] += 1;
        }
    }
    Ok(Histogram { bins })
}

/// Computes one histogram per RGB or BGR channel in storage order.
pub fn histogram_per_channel(input: &ImageBuffer) -> PerceptionResult<[Histogram; 3]> {
    if !matches!(input.pixel_format(), PixelFormat::Rgb8 | PixelFormat::Bgr8) {
        return Err(PerceptionError::UnsupportedFormat {
            operation: "histogram_per_channel",
            format: input.pixel_format().to_string(),
        });
    }
    let mut bins = [[0; 256]; 3];
    for y in 0..input.height() {
        for pixel in input.view().row(y)?.chunks_exact(3) {
            for channel in 0..3 {
                bins[channel][pixel[channel] as usize] += 1;
            }
        }
    }
    Ok(bins.map(|bins| Histogram { bins }))
}

/// Accumulates bin counts from low to high values.
pub fn cumulative_histogram(histogram: &Histogram) -> [u32; 256] {
    let mut output = [0; 256];
    let mut total = 0;
    for (index, value) in histogram.bins.iter().enumerate() {
        total += *value;
        output[index] = total;
    }
    output
}

/// Estimates a value at a percentile in the inclusive zero-to-one range.
pub fn percentile_from_histogram(histogram: &Histogram, quantile: f32) -> PerceptionResult<u8> {
    if !quantile.is_finite() || !(0.0..=1.0).contains(&quantile) {
        return Err(PerceptionError::NumericFailure {
            reason: "histogram percentile must be in [0, 1]".into(),
        });
    }
    let total: u32 = histogram.bins.iter().sum();
    if total == 0 {
        return Err(PerceptionError::InvalidBuffer {
            expected: 1,
            actual: 0,
        });
    }
    let target = (quantile * (total - 1) as f32).round() as u32;
    let mut cumulative = 0;
    for (index, count) in histogram.bins.iter().enumerate() {
        cumulative += *count;
        if cumulative > target {
            return Ok(index as u8);
        }
    }
    Ok(255)
}
