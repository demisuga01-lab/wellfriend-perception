//! Small dense matrices with checked inversion for transform-sized systems.

use core::ops::Mul;

use super::{EPSILON, Vec2, Vec3, near_zero};
use crate::{PerceptionError, PerceptionResult};

/// Two-by-two row-major matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat2 {
    /** Row-major elements. */
    pub values: [[f32; 2]; 2],
}

impl Mat2 {
    /// Identity matrix.
    pub const fn identity() -> Self {
        Self {
            values: [[1.0, 0.0], [0.0, 1.0]],
        }
    }
    /// Determinant.
    pub fn determinant(self) -> f32 {
        self.values[0][0] * self.values[1][1] - self.values[0][1] * self.values[1][0]
    }
    /// Transpose.
    pub fn transpose(self) -> Self {
        Self {
            values: [
                [self.values[0][0], self.values[1][0]],
                [self.values[0][1], self.values[1][1]],
            ],
        }
    }
    /// Checked inverse.
    pub fn inverse(self) -> PerceptionResult<Self> {
        let det = self.determinant();
        if near_zero(det, EPSILON) {
            return Err(PerceptionError::NonInvertibleMatrix);
        }
        Ok(Self {
            values: [
                [self.values[1][1] / det, -self.values[0][1] / det],
                [-self.values[1][0] / det, self.values[0][0] / det],
            ],
        })
    }
}

/// Three-by-three row-major matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat3 {
    /** Row-major elements. */
    pub values: [[f32; 3]; 3],
}

impl Mat3 {
    /// Identity matrix.
    pub const fn identity() -> Self {
        Self {
            values: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
    /// Creates a matrix from row-major values.
    pub const fn new(values: [[f32; 3]; 3]) -> Self {
        Self { values }
    }
    /// Determinant.
    pub fn determinant(self) -> f32 {
        let m = self.values;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }
    /// Transpose.
    pub fn transpose(self) -> Self {
        let m = self.values;
        Self::new([
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ])
    }
    /// Checked adjugate inverse.
    pub fn inverse(self) -> PerceptionResult<Self> {
        let m = self.values;
        let det = self.determinant();
        if !det.is_finite() || near_zero(det, EPSILON) {
            return Err(PerceptionError::NonInvertibleMatrix);
        }
        let cofactors = [
            [
                m[1][1] * m[2][2] - m[1][2] * m[2][1],
                -(m[1][0] * m[2][2] - m[1][2] * m[2][0]),
                m[1][0] * m[2][1] - m[1][1] * m[2][0],
            ],
            [
                -(m[0][1] * m[2][2] - m[0][2] * m[2][1]),
                m[0][0] * m[2][2] - m[0][2] * m[2][0],
                -(m[0][0] * m[2][1] - m[0][1] * m[2][0]),
            ],
            [
                m[0][1] * m[1][2] - m[0][2] * m[1][1],
                -(m[0][0] * m[1][2] - m[0][2] * m[1][0]),
                m[0][0] * m[1][1] - m[0][1] * m[1][0],
            ],
        ];
        Ok(Self::new(cofactors).transpose() * (1.0 / det))
    }
    /// Multiplies a homogeneous vector.
    pub fn mul_vec3(self, vector: Vec3) -> Vec3 {
        let m = self.values;
        Vec3::new(
            m[0][0] * vector.x + m[0][1] * vector.y + m[0][2] * vector.z,
            m[1][0] * vector.x + m[1][1] * vector.y + m[1][2] * vector.z,
            m[2][0] * vector.x + m[2][1] * vector.y + m[2][2] * vector.z,
        )
    }
    /// Converts a 2D point to homogeneous coordinates.
    pub fn to_homogeneous(point: Vec2) -> Vec3 {
        Vec3::new(point.x, point.y, 1.0)
    }
    /// Performs a checked perspective divide.
    pub fn from_homogeneous(vector: Vec3) -> PerceptionResult<Vec2> {
        if !vector.z.is_finite() || near_zero(vector.z, EPSILON) {
            return Err(PerceptionError::NumericFailure {
                reason: "homogeneous coordinate has near-zero w".into(),
            });
        }
        Ok(Vec2::new(vector.x / vector.z, vector.y / vector.z))
    }
}

impl Mul for Mat3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut result = [[0.0; 3]; 3];
        for (row, values) in result.iter_mut().enumerate() {
            for (column, value) in values.iter_mut().enumerate() {
                *value = (0..3)
                    .map(|index| self.values[row][index] * rhs.values[index][column])
                    .sum();
            }
        }
        Self::new(result)
    }
}
impl Mul<f32> for Mat3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        let mut values = self.values;
        for row in &mut values {
            for value in row {
                *value *= rhs;
            }
        }
        Self::new(values)
    }
}

/// Four-by-four row-major matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    /** Row-major elements. */
    pub values: [[f32; 4]; 4],
}

impl Mat4 {
    /// Identity matrix.
    pub const fn identity() -> Self {
        Self {
            values: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
    /// Transpose.
    pub fn transpose(self) -> Self {
        let mut result = [[0.0; 4]; 4];
        for (row, values) in result.iter_mut().enumerate() {
            for (column, value) in values.iter_mut().enumerate() {
                *value = self.values[column][row];
            }
        }
        Self { values: result }
    }
    /// Checked Gauss-Jordan inverse.
    pub fn inverse(self) -> PerceptionResult<Self> {
        let mut augmented = [[0.0; 8]; 4];
        for (row, source) in self.values.iter().enumerate() {
            augmented[row][..4].copy_from_slice(source);
            augmented[row][row + 4] = 1.0;
        }
        for pivot in 0..4 {
            let best = (pivot..4)
                .max_by(|a, b| {
                    augmented[*a][pivot]
                        .abs()
                        .partial_cmp(&augmented[*b][pivot].abs())
                        .unwrap_or(core::cmp::Ordering::Equal)
                })
                .ok_or(PerceptionError::NonInvertibleMatrix)?;
            if near_zero(augmented[best][pivot], EPSILON) {
                return Err(PerceptionError::NonInvertibleMatrix);
            }
            augmented.swap(pivot, best);
            let divisor = augmented[pivot][pivot];
            for value in &mut augmented[pivot] {
                *value /= divisor;
            }
            for row in 0..4 {
                if row != pivot {
                    let factor = augmented[row][pivot];
                    let pivot_values = augmented[pivot];
                    for (value, pivot_value) in augmented[row].iter_mut().zip(pivot_values) {
                        *value -= factor * pivot_value;
                    }
                }
            }
        }
        let mut result = [[0.0; 4]; 4];
        for (row, values) in result.iter_mut().enumerate() {
            values.copy_from_slice(&augmented[row][4..8]);
        }
        Ok(Self { values: result })
    }
}
