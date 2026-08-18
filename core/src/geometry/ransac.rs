//! Deterministic RANSAC line fitting that can later inform generic model adapters.

use super::{LineFit, least_squares_line_fit, point_line_distance};
use crate::{PerceptionError, PerceptionResult, Point2, math::DeterministicRng};

/// Reproducible RANSAC settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RansacConfig {
    /// Number of two-point samples to evaluate.
    pub iterations: usize,
    /// Maximum perpendicular residual for an inlier.
    pub inlier_threshold: f32,
    /// Deterministic sample seed.
    pub seed: u64,
}

impl Default for RansacConfig {
    fn default() -> Self {
        Self {
            iterations: 128,
            inlier_threshold: 1.0,
            seed: 0x5745_4c4c,
        }
    }
}

/// Line fit and its inlier mask after refitting on the winning consensus set.
#[derive(Clone, Debug, PartialEq)]
pub struct RansacLineFit {
    /// Refitted least-squares line.
    pub fit: LineFit,
    /// One entry per input point.
    pub inliers: Vec<bool>,
}

/// Fits a line by repeated two-point hypotheses, scoring, and inlier refitting.
pub fn ransac_line_fit(points: &[Point2], config: RansacConfig) -> PerceptionResult<RansacLineFit> {
    if points.len() < 2 {
        return Err(PerceptionError::InsufficientPoints {
            required: 2,
            actual: points.len(),
        });
    }
    if !config.inlier_threshold.is_finite() || config.inlier_threshold <= 0.0 {
        return Err(PerceptionError::NumericFailure {
            reason: "RANSAC inlier threshold must be finite and positive".into(),
        });
    }
    let mut rng = DeterministicRng::new(config.seed);
    let mut best: Option<(usize, f32, Vec<bool>)> = None;
    for _ in 0..config.iterations.max(1) {
        let first = rng.index(points.len());
        let mut second = rng.index(points.len() - 1);
        if second >= first {
            second += 1;
        }
        let a = points[first];
        let b = points[second];
        if a.distance(b) <= crate::math::EPSILON {
            continue;
        }
        let candidate = crate::Line2 { a, b };
        let inliers: Vec<_> = points
            .iter()
            .map(|point| point_line_distance(*point, candidate) <= config.inlier_threshold)
            .collect();
        let count = inliers.iter().filter(|inside| **inside).count();
        let error = points
            .iter()
            .zip(&inliers)
            .filter_map(|(point, inside)| inside.then_some(point_line_distance(*point, candidate)))
            .sum::<f32>();
        if best.as_ref().is_none_or(|(best_count, best_error, _)| {
            count > *best_count || (count == *best_count && error < *best_error)
        }) {
            best = Some((count, error, inliers));
        }
    }
    let (_, _, inliers) = best.ok_or(PerceptionError::DegenerateGeometry {
        reason: "RANSAC could not sample distinct line points".into(),
    })?;
    let inlier_points: Vec<_> = points
        .iter()
        .zip(&inliers)
        .filter_map(|(point, inside)| inside.then_some(*point))
        .collect();
    if inlier_points.len() < 2 {
        return Err(PerceptionError::DegenerateGeometry {
            reason: "RANSAC consensus has fewer than two points".into(),
        });
    }
    Ok(RansacLineFit {
        fit: least_squares_line_fit(&inlier_points)?,
        inliers,
    })
}

/// Robust line fitting alias for callers that do not need the inlier mask.
pub fn robust_line_fit(points: &[Point2], config: RansacConfig) -> PerceptionResult<LineFit> {
    Ok(ransac_line_fit(points, config)?.fit)
}
