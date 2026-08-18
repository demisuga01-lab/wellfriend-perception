//! Statistics helpers with explicit handling for empty sample sets.

use crate::{PerceptionError, PerceptionResult};

/// Arithmetic mean.
pub fn mean(values: &[f32]) -> PerceptionResult<f32> {
    if values.is_empty() {
        return Err(PerceptionError::InsufficientPoints {
            required: 1,
            actual: 0,
        });
    }
    Ok(values.iter().sum::<f32>() / values.len() as f32)
}
/// Population variance.
pub fn variance(values: &[f32]) -> PerceptionResult<f32> {
    let average = mean(values)?;
    Ok(values
        .iter()
        .map(|value| (value - average).powi(2))
        .sum::<f32>()
        / values.len() as f32)
}
/// Population standard deviation.
pub fn standard_deviation(values: &[f32]) -> PerceptionResult<f32> {
    Ok(variance(values)?.sqrt())
}
/// Median using a copied and sorted sample set.
pub fn median(values: &[f32]) -> PerceptionResult<f32> {
    if values.is_empty() {
        return Err(PerceptionError::InsufficientPoints {
            required: 1,
            actual: 0,
        });
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let middle = sorted.len() / 2;
    Ok(if sorted.len() % 2 == 0 {
        (sorted[middle - 1] + sorted[middle]) * 0.5
    } else {
        sorted[middle]
    })
}
/// Linear-interpolated percentile in the inclusive zero-to-one range.
pub fn percentile(values: &[f32], quantile: f32) -> PerceptionResult<f32> {
    if !(0.0..=1.0).contains(&quantile) || !quantile.is_finite() {
        return Err(PerceptionError::NumericFailure {
            reason: "percentile must be finite and within [0, 1]".into(),
        });
    }
    if values.is_empty() {
        return Err(PerceptionError::InsufficientPoints {
            required: 1,
            actual: 0,
        });
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let index = quantile * (sorted.len() - 1) as f32;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    Ok(sorted[lower] + (sorted[upper] - sorted[lower]) * (index - lower as f32))
}
/// Median absolute deviation around the sample median.
pub fn mad(values: &[f32]) -> PerceptionResult<f32> {
    let center = median(values)?;
    let deviations: Vec<_> = values.iter().map(|value| (value - center).abs()).collect();
    median(&deviations)
}
