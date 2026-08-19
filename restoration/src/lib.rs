//! Condition analysis, deterministic specialist routing, and scalar restoration.
//!
//! The crate provides transparent baselines and contracts for later neural
//! processors.  It deliberately contains no model runtime or model artifact.

pub mod conditions;
pub mod domain_pack;
pub mod filters;
pub mod processors;
pub mod router;
pub mod semantics;

pub use conditions::*;
pub use domain_pack::*;
pub use filters::*;
pub use processors::*;
pub use router::*;
pub use semantics::*;

/// Curated imports for scalar restoration callers.
pub mod prelude {
    pub use crate::{
        ConditionAnalyzerInput, DeviceClass, DocumentFilterGraph, DocumentFilterPreset,
        ScalarConditionAnalyzer, ScalarRestorationProcessor, SpecialistRouter,
    };
}
