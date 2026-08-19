//! Exact four-point homography estimation for domain-neutral planar mapping.

use crate::{
    PerceptionError, PerceptionResult, Point2, Quad, Transform2D, math::solve_linear_system,
};

/// Estimates a projective transform from four source to four target correspondences.
pub fn estimate_homography_4pt(
    source: [Point2; 4],
    target: [Point2; 4],
) -> PerceptionResult<Transform2D> {
    Quad { points: source }.validate()?;
    Quad { points: target }.validate()?;
    let mut matrix = Vec::with_capacity(8);
    let mut vector = Vec::with_capacity(8);
    for (from, to) in source.into_iter().zip(target) {
        matrix.push(vec![
            from.x,
            from.y,
            1.0,
            0.0,
            0.0,
            0.0,
            -to.x * from.x,
            -to.x * from.y,
        ]);
        vector.push(to.x);
        matrix.push(vec![
            0.0,
            0.0,
            0.0,
            from.x,
            from.y,
            1.0,
            -to.y * from.x,
            -to.y * from.y,
        ]);
        vector.push(to.y);
    }
    let h = solve_linear_system(&matrix, &vector)?;
    let transform =
        Transform2D::projective([[h[0], h[1], h[2]], [h[3], h[4], h[5]], [h[6], h[7], 1.0]]);
    if !transform
        .matrix
        .iter()
        .flatten()
        .all(|value| value.is_finite())
    {
        return Err(PerceptionError::NumericFailure {
            reason: "homography solve produced non-finite values".into(),
        });
    }
    Ok(transform)
}

/// Applies a homography to one point.
pub fn apply_homography(transform: Transform2D, point: Point2) -> PerceptionResult<Point2> {
    transform.apply_point(point)
}
/// Inverts a homography.
pub fn invert_homography(transform: Transform2D) -> PerceptionResult<Transform2D> {
    transform.inverse()
}
/// Estimates a transform mapping one quad to another.
pub fn perspective_transform_quad(source: Quad, target: Quad) -> PerceptionResult<Transform2D> {
    estimate_homography_4pt(source.points, target.points)
}
