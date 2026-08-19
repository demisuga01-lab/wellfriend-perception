//! Scalar-first numerical primitives used by geometry and image processing.

pub mod linalg;
pub mod matrix;
pub mod robust;
pub mod scalar;
pub mod stats;
pub mod vector;

pub use linalg::*;
pub use matrix::*;
pub use robust::*;
pub use scalar::*;
pub use stats::*;
pub use vector::*;
