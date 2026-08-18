//! Affine and projective transform construction and application.

use crate::{
    PerceptionResult, Point2, Polygon, Transform2D,
    math::{Mat3, Vec2},
};

impl Transform2D {
    /// Identity transform.
    pub const fn identity() -> Self {
        Self {
            matrix: Mat3::identity().values,
        }
    }
    /// Translation transform.
    pub const fn translation(tx: f32, ty: f32) -> Self {
        Self {
            matrix: [[1.0, 0.0, tx], [0.0, 1.0, ty], [0.0, 0.0, 1.0]],
        }
    }
    /// Axis-aligned scaling transform.
    pub const fn scale(sx: f32, sy: f32) -> Self {
        Self {
            matrix: [[sx, 0.0, 0.0], [0.0, sy, 0.0], [0.0, 0.0, 1.0]],
        }
    }
    /// Counter-clockwise rotation about the origin.
    pub fn rotation(radians: f32) -> Self {
        let (sine, cosine) = radians.sin_cos();
        Self {
            matrix: [[cosine, -sine, 0.0], [sine, cosine, 0.0], [0.0, 0.0, 1.0]],
        }
    }
    /// Creates a transform from affine coefficients.
    pub const fn affine(a: f32, b: f32, tx: f32, c: f32, d: f32, ty: f32) -> Self {
        Self {
            matrix: [[a, b, tx], [c, d, ty], [0.0, 0.0, 1.0]],
        }
    }
    /// Creates a transform from a projective matrix.
    pub const fn projective(matrix: [[f32; 3]; 3]) -> Self {
        Self { matrix }
    }
    /// Composes `self` after `other`, so `self.compose(other).apply(p)` equals `self(other(p))`.
    pub fn compose(self, other: Self) -> Self {
        Self {
            matrix: (Mat3::new(self.matrix) * Mat3::new(other.matrix)).values,
        }
    }
    /// Checked inverse.
    pub fn inverse(self) -> PerceptionResult<Self> {
        Ok(Self {
            matrix: Mat3::new(self.matrix).inverse()?.values,
        })
    }
    /// Applies a homogeneous transform to a point with checked perspective division.
    pub fn apply_point(self, point: Point2) -> PerceptionResult<Point2> {
        let value =
            Mat3::new(self.matrix).mul_vec3(Mat3::to_homogeneous(Vec2::new(point.x, point.y)));
        let result = Mat3::from_homogeneous(value)?;
        Ok(Point2::new(result.x, result.y))
    }
    /// Applies a transform to every polygon vertex.
    pub fn apply_polygon(self, polygon: &Polygon) -> PerceptionResult<Polygon> {
        Ok(Polygon {
            points: polygon
                .points
                .iter()
                .map(|point| self.apply_point(*point))
                .collect::<PerceptionResult<Vec<_>>>()?,
        })
    }
}
