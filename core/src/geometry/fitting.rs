//! Least-squares line fitting using principal-direction covariance analysis.

use crate::{Line2, PerceptionError, PerceptionResult, Point2};

/// Orthogonal least-squares line fit with RMS perpendicular error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineFit {
    /// Infinite line through the fitted centroid.
    pub line: Line2,
    /// Root-mean-square perpendicular residual.
    pub rms_error: f32,
}

/// Fits an orthogonal least-squares line to at least two points.
pub fn least_squares_line_fit(points: &[Point2]) -> PerceptionResult<LineFit> {
    if points.len() < 2 {
        return Err(PerceptionError::InsufficientPoints {
            required: 2,
            actual: points.len(),
        });
    }
    let center = Point2::new(
        points.iter().map(|point| point.x).sum::<f32>() / points.len() as f32,
        points.iter().map(|point| point.y).sum::<f32>() / points.len() as f32,
    );
    let (xx, xy, yy) = points.iter().fold((0.0, 0.0, 0.0), |(xx, xy, yy), point| {
        let dx = point.x - center.x;
        let dy = point.y - center.y;
        (xx + dx * dx, xy + dx * dy, yy + dy * dy)
    });
    if (xx + yy) <= crate::math::EPSILON {
        return Err(PerceptionError::DegenerateGeometry {
            reason: "line points have no measurable spread".into(),
        });
    }
    let angle = 0.5 * (2.0 * xy).atan2(xx - yy);
    let direction = Point2::new(angle.cos(), angle.sin());
    let line = Line2 {
        a: center,
        b: Point2::new(center.x + direction.x, center.y + direction.y),
    };
    let rms_error = (points
        .iter()
        .map(|point| point_line_distance(*point, line).powi(2))
        .sum::<f32>()
        / points.len() as f32)
        .sqrt();
    Ok(LineFit { line, rms_error })
}

/// Alias for the orthogonal least-squares formulation.
pub fn orthogonal_line_fit(points: &[Point2]) -> PerceptionResult<LineFit> {
    least_squares_line_fit(points)
}
/// Perpendicular distance between a point and an infinite line.
pub fn point_line_distance(point: Point2, line: Line2) -> f32 {
    let dx = line.b.x - line.a.x;
    let dy = line.b.y - line.a.y;
    let length = dx.hypot(dy);
    if length <= crate::math::EPSILON {
        return f32::INFINITY;
    }
    ((point.x - line.a.x) * dy - (point.y - line.a.y) * dx).abs() / length
}
