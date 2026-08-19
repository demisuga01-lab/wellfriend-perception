//! Reconstruction contracts and scalar baselines that consume detector output.
//!
//! This crate owns canonical representations and domain-neutral reconstruction
//! seams.  It intentionally does not make a detector decision or apply document
//! restoration; those responsibilities remain in the intelligence and restoration
//! crates respectively.

pub mod contracts;
pub mod document;
pub mod lens;
pub mod quality;
pub mod surface;

pub use contracts::*;
pub use document::*;
pub use lens::*;
pub use quality::*;
pub use surface::*;

/// Curated imports for scalar reconstruction callers.
pub mod prelude {
    pub use crate::{
        AspectRatioPolicy, CanonicalDocument, CropMarginPolicy, OrientationPolicy,
        PlanarDocumentReconstructor, PlanarReconstructionConfig, ReconstructionContext,
        ReconstructionOutput, Reconstructor,
    };
}
