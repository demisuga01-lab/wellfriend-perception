//! Domain-neutral contracts for composable perception and reconstruction pipelines.
//!
//! This crate intentionally contains no document-only concepts. Domain behavior is
//! supplied through [`DomainPack`] implementations.

mod pipeline;
mod traits;
mod types;

pub use pipeline::*;
pub use traits::*;
pub use types::*;
