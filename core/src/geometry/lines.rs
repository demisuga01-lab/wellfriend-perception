//! Line and segment intersection utilities.

use crate::{Line2, Point2, Segment2};

/// Returns true when a point lies on a finite segment within tolerance.
pub fn point_on_segment(point: Point2, start: Point2, end: Point2, epsilon: f32) -> bool {
    let cross = (point.y - start.y) * (end.x - start.x) - (point.x - start.x) * (end.y - start.y);
    if cross.abs() > epsilon {
        return false;
    }
    point.x >= start.x.min(end.x) - epsilon
        && point.x <= start.x.max(end.x) + epsilon
        && point.y >= start.y.min(end.y) - epsilon
        && point.y <= start.y.max(end.y) + epsilon
}

/// Returns an intersection for two non-parallel infinite lines.
pub fn line_intersection(first: Line2, second: Line2) -> Option<Point2> {
    let x1 = first.a.x;
    let y1 = first.a.y;
    let x2 = first.b.x;
    let y2 = first.b.y;
    let x3 = second.a.x;
    let y3 = second.a.y;
    let x4 = second.b.x;
    let y4 = second.b.y;
    let denominator = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    if denominator.abs() <= crate::math::EPSILON {
        return None;
    }
    let determinant_first = x1 * y2 - y1 * x2;
    let determinant_second = x3 * y4 - y3 * x4;
    Some(Point2::new(
        (determinant_first * (x3 - x4) - (x1 - x2) * determinant_second) / denominator,
        (determinant_first * (y3 - y4) - (y1 - y2) * determinant_second) / denominator,
    ))
}

/// Returns an intersection only when it lies on both finite segments.
pub fn segment_intersection(first: Segment2, second: Segment2) -> Option<Point2> {
    let point = line_intersection(
        Line2 {
            a: first.start,
            b: first.end,
        },
        Line2 {
            a: second.start,
            b: second.end,
        },
    )?;
    if point_on_segment(point, first.start, first.end, crate::math::EPSILON)
        && point_on_segment(point, second.start, second.end, crate::math::EPSILON)
    {
        Some(point)
    } else {
        None
    }
}
