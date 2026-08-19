//! Platform-neutral capture-readiness policy derived from perception evidence.

use wellfriend_perception_core::{
    DetectionGeometry, FusionResult, ImageShape, QualityReport, RefinementResult, Score,
    TemporalState,
};

/// Machine-readable capture guidance; rendering belongs in `wellfriend-scan`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureGuidance {
    /// No usable document candidate is present.
    NoDocument,
    /// Page geometry touches or leaves the visible frame.
    DocumentCutOff,
    /// Candidate is too small in the image.
    MoveCloser,
    /// Candidate is too large or borders are not visible.
    MoveFarther,
    /// Geometry track is not yet stable.
    HoldSteady,
    /// Quality analysis reports insufficient sharpness.
    TooBlurry,
    /// Quality analysis reports too little illumination.
    TooDark,
    /// Quality analysis reports excessive illumination.
    TooBright,
    /// Conservative glare baseline is elevated.
    GlareDetected,
    /// Fused/refined evidence is weak.
    LowConfidence,
    /// Detector candidates disagree substantially.
    LowDetectorAgreement,
    /// Current evidence meets capture policy.
    Ready,
}

/// Ordered readiness class supplied to a platform-specific capture controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureReadiness {
    /// Capture should not be offered automatically.
    NotReady,
    /// Input is close but one or more recoverable conditions remain.
    AlmostReady,
    /// Manual capture is justified.
    Ready,
    /// Automatic capture is justified by stable high-confidence evidence.
    CaptureNow,
}

/// Readiness output independent of UI strings or Android camera APIs.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptureReadinessDecision {
    /// Current decision class.
    pub readiness: CaptureReadiness,
    /// Composite bounded readiness score.
    pub score: Score,
    /// Ordered machine-readable guidance.
    pub guidance: Vec<CaptureGuidance>,
    /// Explainable policy diagnostics.
    pub diagnostics: Vec<String>,
}

/// Conservative policy thresholds for auto/manual capture readiness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaptureReadinessConfig {
    /// Lower coverage target for a meaningful page candidate.
    pub minimum_coverage: f32,
    /// Upper coverage boundary after which frame borders are likely missing.
    pub maximum_coverage: f32,
    /// Minimum temporal stability for a ready decision.
    pub minimum_stability: f32,
    /// Minimum fusion confidence for a ready decision.
    pub minimum_confidence: f32,
    /// Score required for `Ready`.
    pub ready_threshold: f32,
    /// Score required for automatic capture.
    pub capture_now_threshold: f32,
}

impl Default for CaptureReadinessConfig {
    fn default() -> Self {
        Self {
            minimum_coverage: 0.20,
            maximum_coverage: 0.93,
            minimum_stability: 0.78,
            minimum_confidence: 0.62,
            ready_threshold: 0.72,
            capture_now_threshold: 0.86,
        }
    }
}

/// Evaluates generic quality, fused geometry, refinement, and temporal evidence.
#[derive(Clone, Debug, Default)]
pub struct CaptureReadinessEngine {
    /// Public deterministic readiness policy.
    pub config: CaptureReadinessConfig,
}

/// Borrowed facts required to make a capture decision.
pub struct CaptureReadinessInput<'a> {
    /// Generic quality analysis and document extensions.
    pub quality: &'a QualityReport,
    /// Detector consensus evidence.
    pub fusion: &'a FusionResult,
    /// High-resolution edge refinement evidence.
    pub refinement: &'a RefinementResult,
    /// Frame-to-frame stability evidence.
    pub temporal: &'a TemporalState,
    /// Dimensions of the current image coordinate system.
    pub image_shape: ImageShape,
}

impl CaptureReadinessEngine {
    /// Makes a machine-readable capture decision without triggering platform capture.
    pub fn evaluate(&self, input: CaptureReadinessInput<'_>) -> CaptureReadinessDecision {
        let mut guidance = Vec::new();
        let quad = match &input.refinement.refined_geometry {
            Some(DetectionGeometry::Quad(quad)) => Some(*quad),
            _ => match &input.fusion.fused_geometry {
                Some(DetectionGeometry::Quad(quad)) => Some(*quad),
                _ => None,
            },
        };
        let Some(quad) = quad else {
            return decision(
                CaptureReadiness::NotReady,
                0.0,
                vec![CaptureGuidance::NoDocument],
                "no fused or refined document quad",
            );
        };
        let coverage = (quad.polygon().area()
            / (input.image_shape.width as f32 * input.image_shape.height as f32).max(1.0))
        .clamp(0.0, 1.0);
        let is_cut_off = input
            .quality
            .warnings
            .iter()
            .any(|value| value == "document_cut_off")
            || quad.points.iter().any(|point| {
                point.x < 0.0
                    || point.y < 0.0
                    || point.x >= input.image_shape.width as f32
                    || point.y >= input.image_shape.height as f32
            });
        if is_cut_off {
            guidance.push(CaptureGuidance::DocumentCutOff);
        }
        if coverage < self.config.minimum_coverage {
            guidance.push(CaptureGuidance::MoveCloser);
        }
        if coverage > self.config.maximum_coverage {
            guidance.push(CaptureGuidance::MoveFarther);
        }
        append_quality_guidance(&mut guidance, input.quality);
        if !input.temporal.stable
            || input.temporal.stability_score.value() < self.config.minimum_stability
        {
            guidance.push(CaptureGuidance::HoldSteady);
        }
        if input.fusion.confidence.value() < self.config.minimum_confidence
            || input.refinement.confidence.value() < self.config.minimum_confidence
        {
            guidance.push(CaptureGuidance::LowConfidence);
        }
        if input.fusion.disagreement_score.value() > 0.35 {
            guidance.push(CaptureGuidance::LowDetectorAgreement);
        }
        let exposure = metric(input.quality, "mean_luminance", 0.5);
        let blur = metric(input.quality, "blur_laplacian_variance", 0.5).max(metric(
            input.quality,
            "blur_tenengrad_energy",
            0.5,
        ));
        let contrast = metric(input.quality, "contrast_percentile_range", 0.5);
        let coverage_score = ((coverage - self.config.minimum_coverage)
            / (self.config.maximum_coverage - self.config.minimum_coverage).max(0.01))
        .clamp(0.0, 1.0);
        let score = (0.17 * coverage_score
            + 0.13 * exposure
            + 0.13 * blur
            + 0.09 * contrast
            + 0.20 * input.fusion.confidence.value()
            + 0.10 * input.refinement.confidence.value()
            + 0.14 * input.temporal.stability_score.value()
            + 0.04 * (1.0 - input.fusion.disagreement_score.value()))
        .clamp(0.0, 1.0);
        let blocking = guidance.iter().any(|item| {
            matches!(
                item,
                CaptureGuidance::NoDocument
                    | CaptureGuidance::DocumentCutOff
                    | CaptureGuidance::TooBlurry
                    | CaptureGuidance::TooDark
                    | CaptureGuidance::TooBright
                    | CaptureGuidance::HoldSteady
            )
        });
        let readiness = if blocking || score < 0.45 {
            CaptureReadiness::NotReady
        } else if score < self.config.ready_threshold || !guidance.is_empty() {
            CaptureReadiness::AlmostReady
        } else if score >= self.config.capture_now_threshold {
            guidance.push(CaptureGuidance::Ready);
            CaptureReadiness::CaptureNow
        } else {
            guidance.push(CaptureGuidance::Ready);
            CaptureReadiness::Ready
        };
        decision(
            readiness,
            score,
            guidance,
            "quality + consensus + refinement + temporal policy",
        )
    }
}

fn metric(report: &QualityReport, name: &str, default: f32) -> f32 {
    report
        .metrics
        .get(name)
        .map(|metric| metric.normalized_score.value())
        .unwrap_or(default)
}

fn append_quality_guidance(guidance: &mut Vec<CaptureGuidance>, report: &QualityReport) {
    for warning in &report.warnings {
        let item = match warning.as_str() {
            "too_blurry" => Some(CaptureGuidance::TooBlurry),
            "too_dark" => Some(CaptureGuidance::TooDark),
            "too_bright" => Some(CaptureGuidance::TooBright),
            "glare_detected" => Some(CaptureGuidance::GlareDetected),
            _ => None,
        };
        if let Some(item) = item {
            guidance.push(item);
        }
    }
}

fn decision(
    readiness: CaptureReadiness,
    score: f32,
    guidance: Vec<CaptureGuidance>,
    diagnostic: &str,
) -> CaptureReadinessDecision {
    CaptureReadinessDecision {
        readiness,
        score: Score::new(score.clamp(0.0, 1.0)).unwrap_or_default(),
        guidance,
        diagnostics: vec![diagnostic.into()],
    }
}
