//! Domain-neutral contracts for composable perception and reconstruction pipelines.
//!
//! This crate intentionally contains no document-only concepts. Domain behavior is
//! supplied through [`DomainPack`] implementations.

pub mod benchmarks;
pub mod boundary;
mod diagnostics;
mod error;
pub mod geometry;
pub mod math;
mod pipeline;
pub mod prelude;
mod traits;
mod types;

pub use boundary::*;
pub use diagnostics::*;
pub use error::*;
pub use pipeline::*;
pub use traits::*;
pub use types::*;
