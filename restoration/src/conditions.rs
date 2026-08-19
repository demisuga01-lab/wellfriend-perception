//! Condition analysis translates evidence into explicit processor-relevant signals.

use std::collections::BTreeMap;

use wellfriend_perception_core::{
    Confidence, FusionResult, QualityReport, RefinementResult, Score, TemporalState,
};
use wellfriend_perception_reconstruction::ReconstructionQualityReport;

use crate::ProcessorId;

/// Stable generic and document-specific condition identifiers.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConditionKind {
    /// Blur remains after capture/reconstruction.
    Blur,
    /// High-frequency residual noise.
    Noise,
    /// Insufficient illumination.
    Underexposure,
    /// Clipped illumination.
    Overexposure,
    /// Insufficient text/background separation.
    LowContrast,
    /// Bright saturated low-texture region.
    Glare,
    /// Missing image area or obstruction.
    Occlusion,
    /// Frame motion or instability.
    Motion,
    /// Detector/fusion/refinement geometry disagreement.
    GeometryUncertainty,
    /// Risk introduced by canonical reconstruction.
    ReconstructionDistortion,
    /// Document-local illumination shadow estimate.
    Shadow,
    /// Document-local color-cast placeholder.
    YellowLighting,
    /// Document-local low-ink placeholder.
    FadedText,
    /// Document-local background contamination placeholder.
    BackgroundDirty,
    /// Curved-page indicator.
    PageCurvature,
    /// Bound-book gutter placeholder.
    PageGutter,
    /// Receipt-specific contrast indicator.
    ReceiptLowContrast,
    /// Safety signal that discourages aggressive filters on handwriting.
    HandwritingSensitive,
    /// Explicit future semantic sensitivity seam.
    PhotoRegionSensitivePlaceholder,
    /// Explicit future semantic sensitivity seam.
    SignatureSensitivePlaceholder,
}

impl ConditionKind {
    /// Stable machine-readable identifier.
    pub const fn id(&self) -> &'static str {
        match self {
            Self::Blur => "blur",
            Self::Noise => "noise",
            Self::Underexposure => "underexposure",
            Self::Overexposure => "overexposure",
            Self::LowContrast => "low_contrast",
            Self::Glare => "glare",
            Self::Occlusion => "occlusion",
            Self::Motion => "motion",
            Self::GeometryUncertainty => "geometry_uncertainty",
            Self::ReconstructionDistortion => "reconstruction_distortion",
            Self::Shadow => "shadow",
            Self::YellowLighting => "yellow_lighting",
            Self::FadedText => "faded_text",
            Self::BackgroundDirty => "background_dirty",
            Self::PageCurvature => "page_curvature",
            Self::PageGutter => "page_gutter",
            Self::ReceiptLowContrast => "receipt_low_contrast",
            Self::HandwritingSensitive => "handwriting_sensitive",
            Self::PhotoRegionSensitivePlaceholder => "photo_region_sensitive_placeholder",
            Self::SignatureSensitivePlaceholder => "signature_sensitive_placeholder",
        }
    }
}

/// Evidence for one routing condition.
#[derive(Clone, Debug, PartialEq)]
pub struct ConditionEvidence {
    /// Bounded severity; higher means the condition is more relevant.
    pub score: Score,
    /// Reliability of this condition evidence.
    pub confidence: Confidence,
    /// Human- and machine-readable bounded evidence notes.
    pub evidence: Vec<String>,
    /// Producer identifiers, preserving provenance.
    pub sources: Vec<String>,
    /// Processors that may help when routing permits them.
    pub recommended_processors: Vec<ProcessorId>,
    /// Explicit limitations such as uncalibrated placeholders.
    pub diagnostics: Vec<String>,
}

/// Typed condition collection used by the specialist router.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConditionVector {
    /// Condition evidence keyed by stable condition kind.
    pub entries: BTreeMap<ConditionKind, ConditionEvidence>,
}

impl ConditionVector {
    /// Returns a bounded severity or zero when the condition is absent.
    pub fn score(&self, condition: ConditionKind) -> f32 {
        self.entries
            .get(&condition)
            .map(|evidence| evidence.score.value())
            .unwrap_or(0.0)
    }

    /// Adds or replaces one condition after callers have constructed valid bounds.
    pub fn insert(&mut self, kind: ConditionKind, evidence: ConditionEvidence) {
        self.entries.insert(kind, evidence);
    }
}

/// Inputs available after MP3 intelligence and MP4 reconstruction.
#[derive(Clone, Debug, PartialEq)]
pub struct ConditionAnalyzerInput<'a> {
    /// Generic scalar image-quality result.
    pub quality: &'a QualityReport,
    /// Candidate agreement and provenance result.
    pub fusion: Option<&'a FusionResult>,
    /// High-resolution geometry refinement result.
    pub refinement: Option<&'a RefinementResult>,
    /// Temporal stability result.
    pub temporal: Option<&'a TemporalState>,
    /// Canonical-page quality and distortion report.
    pub reconstruction: Option<&'a ReconstructionQualityReport>,
    /// Optional document metadata that a domain pack may provide.
    pub domain_metadata: BTreeMap<String, String>,
}

/// Deterministic scalar condition analyzer; it does not claim trained classification.
#[derive(Clone, Debug, Default)]
pub struct ScalarConditionAnalyzer;

impl ScalarConditionAnalyzer {
    /// Builds a condition vector from available quality, geometry, temporal, and reconstruction evidence.
    pub fn analyze(&self, input: &ConditionAnalyzerInput<'_>) -> ConditionVector {
        let mut conditions = ConditionVector::default();
        let metric = |name: &str| input.quality.metrics.get(name);
        insert(
            &mut conditions,
            ConditionKind::Blur,
            severity_from_quality(metric("blur_laplacian_variance"), true),
            "quality:blur_laplacian_variance",
            vec![ProcessorId::new("unsharp")],
        );
        insert(
            &mut conditions,
            ConditionKind::Noise,
            severity_from_quality(metric("noise_residual"), true),
            "quality:noise_residual",
            vec![ProcessorId::new("denoise")],
        );
        insert(
            &mut conditions,
            ConditionKind::Underexposure,
            raw_metric(metric("underexposed_fraction")),
            "quality:underexposed_fraction",
            vec![
                ProcessorId::new("brightness_contrast"),
                ProcessorId::new("gamma"),
            ],
        );
        insert(
            &mut conditions,
            ConditionKind::Overexposure,
            raw_metric(metric("overexposed_fraction")),
            "quality:overexposed_fraction",
            vec![ProcessorId::new("brightness_contrast")],
        );
        insert(
            &mut conditions,
            ConditionKind::LowContrast,
            severity_from_quality(metric("contrast_percentile_range"), true),
            "quality:contrast_percentile_range",
            vec![
                ProcessorId::new("brightness_contrast"),
                ProcessorId::new("background_normalization"),
            ],
        );
        insert(
            &mut conditions,
            ConditionKind::Glare,
            raw_metric(metric("glare_likelihood")),
            "quality:glare_likelihood",
            Vec::new(),
        );
        if let Some(temporal) = input.temporal {
            let motion = (1.0 - temporal.stability_score.value()).clamp(0.0, 1.0);
            insert_raw(
                &mut conditions,
                ConditionKind::Motion,
                motion,
                temporal.confidence,
                "temporal:stability_score",
                Vec::new(),
                Vec::new(),
            );
        }
        if let Some(fusion) = input.fusion {
            let fusion_uncertainty =
                (1.0 - fusion.confidence.value()).max(fusion.disagreement_score.value());
            let refinement_penalty = input
                .refinement
                .map(|result| 1.0 - result.confidence.value())
                .unwrap_or(0.0);
            insert_raw(
                &mut conditions,
                ConditionKind::GeometryUncertainty,
                fusion_uncertainty.max(refinement_penalty).clamp(0.0, 1.0),
                fusion.confidence,
                "fusion/refinement:geometry_agreement",
                Vec::new(),
                Vec::new(),
            );
        }
        if let Some(reconstruction) = input.reconstruction {
            insert_raw(
                &mut conditions,
                ConditionKind::ReconstructionDistortion,
                reconstruction
                    .reconstruction
                    .warp_stretch_risk
                    .value()
                    .max(reconstruction.reconstruction.aspect_distortion_risk.value()),
                Confidence::new(0.65).expect("fixed bounded confidence"),
                "reconstruction:warp_and_aspect_risk",
                Vec::new(),
                Vec::new(),
            );
        }
        insert_document_placeholders(&mut conditions, input);
        conditions
    }
}

fn severity_from_quality(
    metric: Option<&wellfriend_perception_core::QualityMeasurement>,
    invert_normalized: bool,
) -> f32 {
    metric
        .map(|measurement| {
            if invert_normalized {
                1.0 - measurement.normalized_score.value()
            } else {
                measurement.normalized_score.value()
            }
        })
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

fn raw_metric(metric: Option<&wellfriend_perception_core::QualityMeasurement>) -> f32 {
    metric
        .map(|measurement| measurement.raw_value.clamp(0.0, 1.0))
        .unwrap_or(0.0)
}

fn insert(
    conditions: &mut ConditionVector,
    kind: ConditionKind,
    score: f32,
    source: &str,
    processors: Vec<ProcessorId>,
) {
    insert_raw(
        conditions,
        kind,
        score,
        Confidence::new(0.65).expect("fixed bounded confidence"),
        source,
        processors,
        Vec::new(),
    );
}

fn insert_raw(
    conditions: &mut ConditionVector,
    kind: ConditionKind,
    score: f32,
    confidence: Confidence,
    source: &str,
    processors: Vec<ProcessorId>,
    diagnostics: Vec<String>,
) {
    conditions.insert(
        kind,
        ConditionEvidence {
            score: Score::new(score.clamp(0.0, 1.0)).expect("clamped condition score"),
            confidence,
            evidence: vec![format!("severity={:.3}", score.clamp(0.0, 1.0))],
            sources: vec![source.into()],
            recommended_processors: processors,
            diagnostics,
        },
    );
}

fn insert_document_placeholders(
    conditions: &mut ConditionVector,
    input: &ConditionAnalyzerInput<'_>,
) {
    let shadow = input
        .quality
        .metrics
        .get("document_shadow_likelihood_baseline")
        .map(|metric| metric.raw_value.clamp(0.0, 1.0))
        .unwrap_or(0.0);
    insert_raw(
        conditions,
        ConditionKind::Shadow,
        shadow,
        Confidence::new(0.3).expect("fixed bounded confidence"),
        "document_quality:shadow_likelihood_baseline",
        vec![ProcessorId::new("background_normalization")],
        vec!["document shadow estimate is a scalar uncalibrated baseline".into()],
    );
    let curvature = input
        .domain_metadata
        .get("page_curvature_likelihood")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    insert_raw(
        conditions,
        ConditionKind::PageCurvature,
        curvature,
        Confidence::new(0.0).expect("fixed bounded confidence"),
        "document_metadata:page_curvature_likelihood",
        Vec::new(),
        vec!["page curvature requires surface evidence; scalar placeholder is uncalibrated".into()],
    );
    for kind in [
        ConditionKind::YellowLighting,
        ConditionKind::FadedText,
        ConditionKind::BackgroundDirty,
        ConditionKind::PageGutter,
        ConditionKind::ReceiptLowContrast,
        ConditionKind::HandwritingSensitive,
        ConditionKind::PhotoRegionSensitivePlaceholder,
        ConditionKind::SignatureSensitivePlaceholder,
    ] {
        insert_raw(
            conditions,
            kind,
            0.0,
            Confidence::new(0.0).expect("fixed bounded confidence"),
            "document_placeholder",
            Vec::new(),
            vec![
                "unimplemented document condition placeholder; no trained detector is claimed"
                    .into(),
            ],
        );
    }
}
