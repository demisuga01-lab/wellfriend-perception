//! Small scalar helpers that make numerical preconditions explicit.

/// Default tolerance used for small matrix and geometry checks.
pub const EPSILON: f32 = 1.0e-6;

/// Returns whether a value is finite and near zero under the supplied tolerance.
pub fn near_zero(value: f32, epsilon: f32) -> bool {
    value.is_finite() && value.abs() <= epsilon.abs()
}

/// Clamps finite values into an inclusive range.
pub fn clamp(value: f32, minimum: f32, maximum: f32) -> f32 {
    value.clamp(minimum, maximum)
}
