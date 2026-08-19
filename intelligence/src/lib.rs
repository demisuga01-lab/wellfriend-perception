//! Perception intelligence built on Wellfriend's checked core and scalar image layers.
//!
//! This crate owns quality analysis, detector contracts, fusion, refinement,
//! temporal smoothing, capture readiness, and the document reference baseline.
//! It deliberately contains no platform capture, model runtime, OCR, or UI code.

pub mod benchmarks;
pub mod detection;
pub mod domains;
pub mod fusion;
pub mod quality;
pub mod readiness;
pub mod refinement;
pub mod temporal;

/// Curated imports for an end-to-end MP3 perception intelligence pipeline.
pub mod prelude {
    pub use crate::{
        detection::{DetectorInput, DetectorOutput, ModelTask, PerceptionDetector},
        fusion::QuadFusionEngine,
        quality::ScalarQualityAnalyzer,
        readiness::{CaptureReadinessDecision, CaptureReadinessEngine},
        refinement::QuadRefiner,
        temporal::QuadTemporalTracker,
    };
}
