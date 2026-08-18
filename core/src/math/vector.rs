//! Fixed-size vectors used by the generic geometry layer.

use core::ops::{Add, Div, Mul, Neg, Sub};

use crate::{PerceptionError, PerceptionResult};

/// Two-component floating-point vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}

impl Vec2 {
    /// Creates a vector from components.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    /// Returns the dot product.
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    /// Returns the signed 2D cross product magnitude.
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
    /// Returns squared Euclidean norm.
    pub fn norm_squared(self) -> f32 {
        self.dot(self)
    }
    /// Returns Euclidean norm.
    pub fn norm(self) -> f32 {
        self.norm_squared().sqrt()
    }
    /// Returns a unit vector or a numeric error for zero/non-finite vectors.
    pub fn normalized(self) -> PerceptionResult<Self> {
        let norm = self.norm();
        if !norm.is_finite() || norm <= crate::math::EPSILON {
            return Err(PerceptionError::NumericFailure {
                reason: "cannot normalize a zero or non-finite Vec2".into(),
            });
        }
        Ok(self / norm)
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}
impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}
impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

/// Three-component floating-point vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}

impl Vec3 {
    /// Creates a vector from components.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    /// Returns the dot product.
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    /// Returns the 3D cross product.
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    /// Returns Euclidean norm.
    pub fn norm(self) -> f32 {
        self.dot(self).sqrt()
    }
    /// Returns a unit vector or an error for invalid vectors.
    pub fn normalized(self) -> PerceptionResult<Self> {
        let norm = self.norm();
        if !norm.is_finite() || norm <= crate::math::EPSILON {
            return Err(PerceptionError::NumericFailure {
                reason: "cannot normalize a zero or non-finite Vec3".into(),
            });
        }
        Ok(self / norm)
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}
impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}
