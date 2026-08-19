//! Explicit reconstruction input, output, artifact, and family contracts.

use std::collections::BTreeMap;

use wellfriend_perception_core::{
    Confidence, DenseWarpField, ImageBuffer, PerceptionResult, Point2, Score,
};

use crate::SurfaceModel;

/// Reconstruction family selected by a domain pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconstructionFamily {
    /// A 2D planar mapping such as a flat page or card.
    Planar,
    /// A 2.5D surface mapping such as a curved page.
    Surface,
    /// A 3D voxel or volumetric reconstruction.
    Volumetric,
    /// A geographic-coordinate reconstruction.
    Geospatial,
    /// A camera-pose and scene-surface reconstruction.
    Photogrammetric,
}

/// A named, observable step in a reconstruction trace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconstructionStage {
    /// Input shape and geometry validation.
    InputValidation,
    /// Lens correction or its documented no-op seam.
    LensCorrection,
    /// Planar homography estimation and resampling.
    PlanarWarp,
    /// Dense or mesh surface resampling.
    SurfaceWarp,
    /// Post-reconstruction quality measurement.
    QualityEvaluation,
}

/// Human- and machine-readable reconstruction evidence.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReconstructionDiagnostics {
    /// Executed or deliberately skipped stages in order.
    pub stages: Vec<ReconstructionStage>,
    /// Stable diagnostic identifiers or bounded descriptions.
    pub messages: Vec<String>,
    /// Extension values that remain domain-neutral at this seam.
    pub attributes: BTreeMap<String, String>,
}

/// Bounded confidence with explanations of its inputs and limits.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReconstructionConfidence {
    /// Bounded implementation confidence, not a calibrated probability.
    pub value: Confidence,
    /// Reasons that support or limit this value.
    pub diagnostics: Vec<String>,
}

/// Generic reconstruction quality values that every canonical artifact can carry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReconstructionQuality {
    /// Quality score after resampling when a 2D image exists.
    pub output_quality_score: Score,
    /// Fraction of intended source coverage lost by an explicit policy.
    pub coverage_loss: Score,
    /// Conservative normalized warp stretch risk.
    pub warp_stretch_risk: Score,
    /// Conservative normalized aspect-distortion risk.
    pub aspect_distortion_risk: Score,
    /// Additional domain-neutral diagnostics.
    pub diagnostics: Vec<String>,
}

/// Material artifact emitted by a reconstruction stage.
#[derive(Clone, Debug, PartialEq)]
pub enum ReconstructionArtifact {
    /// A canonical 2D image.
    Image(ImageBuffer),
    /// A validated output-to-source coordinate field.
    DenseWarp(DenseWarpField),
    /// A generic sampled surface model.
    Surface(SurfaceModel),
}

/// Canonical geometry independent of a document-only representation.
#[derive(Clone, Debug, PartialEq)]
pub enum CanonicalGeometry {
    /// Pixel coordinates in a canonical plane.
    Planar {
        /// Output width in pixels.
        width: u32,
        /// Output height in pixels.
        height: u32,
        /// Origin and axis declaration for consumers.
        origin: Point2,
    },
    /// A parameterized surface model.
    Surface(SurfaceModel),
    /// Correct contract seam; no MP4 volume implementation is implied.
    VolumetricPlaceholder,
    /// Correct contract seam; no MP4 geospatial implementation is implied.
    GeospatialPlaceholder,
    /// Correct contract seam; no MP4 photogrammetry implementation is implied.
    PhotogrammetricPlaceholder,
}

/// A domain-neutral canonical result that can contain one or more artifacts.
#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalRepresentation {
    /// Family that governs interpretation of geometry and artifacts.
    pub family: ReconstructionFamily,
    /// Canonical geometric coordinate definition.
    pub geometry: CanonicalGeometry,
    /// Produced material artifacts.
    pub artifacts: Vec<ReconstructionArtifact>,
    /// Traceable reconstruction decisions.
    pub diagnostics: ReconstructionDiagnostics,
}

/// Generic input wrapper for a reconstruction stage.
#[derive(Clone, Debug, PartialEq)]
pub struct ReconstructionInput {
    /// The observation image used by baseline planar and surface paths.
    pub image: ImageBuffer,
    /// Extensible input metadata without document-specific keys in core APIs.
    pub attributes: BTreeMap<String, String>,
}

/// Generic output wrapper produced by a reconstructor.
#[derive(Clone, Debug, PartialEq)]
pub struct ReconstructionOutput {
    /// Canonical result materialized by the stage.
    pub canonical: CanonicalRepresentation,
    /// Bounded implementation confidence with limitations.
    pub confidence: ReconstructionConfidence,
    /// Scalar quality values available before restoration.
    pub quality: ReconstructionQuality,
    /// Ordered reconstruction trace.
    pub diagnostics: ReconstructionDiagnostics,
}

/// Context supplied by an orchestrator without binding a reconstruction to a UI.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReconstructionContext {
    /// Domain identifier selected by the runtime.
    pub domain_id: Option<String>,
    /// Stable pipeline or benchmark run identifier.
    pub run_id: Option<String>,
    /// Domain-neutral context extensions.
    pub attributes: BTreeMap<String, String>,
}

/// A typed reconstructor boundary.
///
/// Typed input and output keep a planar document reconstructor separate from
/// eventual volume, geospatial, and photogrammetric implementations.
pub trait Reconstructor {
    /// Validated input for this reconstruction family.
    type Input;
    /// Typed output for this reconstruction family.
    type Output;

    /// Produces a canonical representation or a structured perception error.
    fn reconstruct(
        &self,
        input: &Self::Input,
        context: &ReconstructionContext,
    ) -> PerceptionResult<Self::Output>;
}

/// Describes a placeholder family without claiming an implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconstructionPlaceholder {
    /// Family that a future implementation will own.
    pub family: ReconstructionFamily,
    /// Explicit non-production limitation.
    pub diagnostic: String,
}

impl ReconstructionPlaceholder {
    /// Creates a deliberate interface-only family registration.
    pub fn new(family: ReconstructionFamily, diagnostic: impl Into<String>) -> Self {
        Self {
            family,
            diagnostic: diagnostic.into(),
        }
    }
}
