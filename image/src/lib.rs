//! Scalar-first image operations built on the checked core image buffer model.
//!
//! MP2 intentionally favors explicit formats, predictable borders, and correctness
//! over SIMD or accelerator optimization. All image algorithms here are domain-neutral.

mod color;
mod filtering;
mod histogram;
mod operations;
mod resize;
mod threshold;

pub use color::*;
pub use filtering::*;
pub use histogram::*;
pub use operations::*;
pub use resize::*;
pub use threshold::*;
