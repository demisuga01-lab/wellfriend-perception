//! Detector contracts, typed document candidates, and runtime-agnostic model seams.

use wellfriend_perception_core::{
    Confidence, DetectionCandidate, DetectionConfidence, DetectionDiagnostics, DetectionGeometry,
    DetectionSet, DetectionSource, DetectorCapabilities, DetectorConfig, ImageBuffer,
    PerceptionError, PerceptionResult, Quad, Score, Uncertainty,
};

/// Borrowed detector input that keeps image ownership with the caller.
#[derive(Clone, Copy, Debug)]
pub struct DetectorInput<'a> {
    /// Image to inspect.
    pub image: &'a ImageBuffer,
    /// Optional monotonic source frame index.
    pub frame_index: Option<u64>,
}

/// Output from one independently attributable detector execution.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DetectorOutput {
    /// Candidates and set-level provenance.
    pub detections: DetectionSet,
    /// Capability snapshot used by the executing detector.
    pub capabilities: DetectorCapabilities,
    /// Execution diagnostics that do not apply to a single candidate.
    pub diagnostics: Vec<String>,
}

/// Model tasks recognized by production adapters without hardcoding a model runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelTask {
    /// Produces a foreground or region mask.
    Segmentation,
    /// Produces page or object corner coordinates.
    CornerRegression,
    /// Predicts image or capture quality signals.
    QualityPrediction,
    /// Produces restoration/cleanup masks.
    CleanupMask,
    /// Future artifact task declared by its manifest.
    Custom(String),
}

impl ModelTask {
    /// Stable artifact task label.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Segmentation => "segmentation",
            Self::CornerRegression => "corner-regression",
            Self::QualityPrediction => "quality-prediction",
            Self::CleanupMask => "cleanup-mask",
            Self::Custom(value) => value,
        }
    }
}

/// Interface implemented by classical, temporal, manual, and model-backed detectors.
pub trait PerceptionDetector {
    /// Stable detector identifier.
    fn id(&self) -> &str;
    /// Declared output and runtime constraints.
    fn capabilities(&self) -> DetectorCapabilities;
    /// Produces independently attributable candidates.
    fn detect(&self, input: DetectorInput<'_>) -> PerceptionResult<DetectorOutput>;
}

/// Extracts a validated quad from a generic candidate.
pub fn candidate_quad(candidate: &DetectionCandidate) -> PerceptionResult<Quad> {
    match &candidate.geometry_payload {
        Some(DetectionGeometry::Quad(quad)) => {
            quad.validate()?;
            Ok(*quad)
        }
        Some(DetectionGeometry::Polygon(polygon)) => polygon_as_quad(polygon),
        None => candidate
            .geometry
            .as_ref()
            .ok_or(PerceptionError::UnsupportedOperation {
                operation: "candidate does not carry quad geometry",
            })
            .and_then(polygon_as_quad),
        Some(_) => Err(PerceptionError::UnsupportedOperation {
            operation: "candidate geometry is not a quad",
        }),
    }
}

fn polygon_as_quad(polygon: &wellfriend_perception_core::Polygon) -> PerceptionResult<Quad> {
    let points: [wellfriend_perception_core::Point2; 4] = polygon
        .points
        .clone()
        .try_into()
        .map_err(|points: Vec<_>| PerceptionError::InsufficientPoints {
            required: 4,
            actual: points.len(),
        })?;
    let quad = Quad { points };
    quad.validate()?;
    Ok(quad)
}

/// Creates a document-domain quad candidate while retaining generic geometry payload.
pub fn document_quad_candidate(
    source: DetectionSource,
    quad: Quad,
    score: f32,
    detector_id: impl Into<String>,
) -> PerceptionResult<DetectionCandidate> {
    quad.validate()?;
    let score = Score::new(score)?;
    let confidence = Confidence::new(score.value())?;
    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert("domain".into(), "document".into());
    attributes.insert("detector_id".into(), detector_id.into());
    attributes.insert("confidence_kind".into(), "heuristic".into());
    Ok(DetectionCandidate {
        kind: "quad".into(),
        source,
        confidence: DetectionConfidence {
            score: confidence,
            lower: Confidence::new((confidence.value() - 0.15).max(0.0))?,
            upper: Confidence::new((confidence.value() + 0.15).min(1.0))?,
        },
        geometry: Some(quad.polygon()),
        geometry_payload: Some(DetectionGeometry::Quad(quad)),
        score,
        uncertainty: Uncertainty::with_variance((1.0 - confidence.value()).powi(2))?,
        diagnostics: DetectionDiagnostics::default(),
        attributes,
    })
}

/// Validates and wraps user-provided geometry for the fusion path.
pub fn manual_quad_candidate(quad: Quad) -> PerceptionResult<DetectionCandidate> {
    let mut candidate =
        document_quad_candidate(DetectionSource::Manual, quad, 1.0, "manual-correction")?;
    candidate
        .attributes
        .insert("manual_override".into(), "true".into());
    Ok(candidate)
}

/// A model execution runtime that can later be fulfilled by ONNX, LiteRT, WASM, or native code.
pub trait ModelRuntime {
    /// Runtime identifier, such as `onnxruntime`, `litert`, or `wasm`.
    fn runtime_id(&self) -> &str;
    /// Executes an already-validated model artifact without importing Python.
    fn infer(&self, input: DetectorInput<'_>) -> PerceptionResult<DetectionSet>;
}

/// Model adapter contract separating artifact metadata from a concrete inference runtime.
pub trait ModelDetectorAdapter: PerceptionDetector {
    /// Model artifact identifier from the model-artifact contract.
    fn artifact_id(&self) -> &str;
    /// Declared artifact task, for example `segmentation` or `corner-regression`.
    fn task(&self) -> &ModelTask;
    /// Runtime class required by the adapter.
    fn runtime_class(&self) -> &str;
}

/// Runtime-backed adapter that adds declared model provenance to its output.
pub struct RuntimeModelDetector<R> {
    /// Artifact metadata reference; no weights or arbitrary Python are loaded here.
    pub artifact_id: String,
    /// Artifact task declared in `manifest.json`.
    pub task: ModelTask,
    /// Underlying native, WASM, LiteRT, or test runtime.
    pub runtime: R,
    /// Adapter configuration.
    pub config: DetectorConfig,
}

impl<R: ModelRuntime> PerceptionDetector for RuntimeModelDetector<R> {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn capabilities(&self) -> DetectorCapabilities {
        DetectorCapabilities {
            geometry_kinds: vec!["quad".into(), "mask".into(), "point".into()],
            model_backed: true,
            accepts_manual_geometry: false,
            supported_runtime_classes: vec![self.runtime.runtime_id().into()],
        }
    }

    fn detect(&self, input: DetectorInput<'_>) -> PerceptionResult<DetectorOutput> {
        let mut detections = self.runtime.infer(input)?;
        detections.detector_id = Some(self.id().into());
        for candidate in &mut detections.candidates {
            candidate
                .attributes
                .insert("model_artifact".into(), self.artifact_id.clone());
            candidate
                .attributes
                .insert("model_task".into(), self.task.as_str().into());
        }
        Ok(DetectorOutput {
            detections,
            capabilities: self.capabilities(),
            diagnostics: vec!["model adapter executed through declared runtime only".into()],
        })
    }
}

impl<R: ModelRuntime> ModelDetectorAdapter for RuntimeModelDetector<R> {
    fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    fn task(&self) -> &ModelTask {
        &self.task
    }

    fn runtime_class(&self) -> &str {
        self.runtime.runtime_id()
    }
}

#[cfg(test)]
pub(crate) struct MockModelDetector;

#[cfg(test)]
impl ModelRuntime for MockModelDetector {
    fn runtime_id(&self) -> &str {
        "test-mock"
    }

    fn infer(&self, _: DetectorInput<'_>) -> PerceptionResult<DetectionSet> {
        Ok(DetectionSet {
            candidates: Vec::new(),
            detector_id: Some("test-mock".into()),
            diagnostics: vec!["test-only mock model output".into()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DetectorInput, MockModelDetector, ModelTask, PerceptionDetector, RuntimeModelDetector,
    };
    use wellfriend_perception_core::{DetectorConfig, ImageBuffer, PixelFormat};

    #[test]
    fn test_only_mock_runtime_stays_behind_adapter_contract() {
        let adapter = RuntimeModelDetector {
            artifact_id: "test-artifact".into(),
            task: ModelTask::Segmentation,
            runtime: MockModelDetector,
            config: DetectorConfig {
                id: "test-model-adapter".into(),
                max_candidates: 1,
                attributes: Default::default(),
            },
        };
        let image = ImageBuffer::new(1, 1, PixelFormat::Gray8, vec![0]).unwrap();
        let output = adapter
            .detect(DetectorInput {
                image: &image,
                frame_index: None,
            })
            .unwrap();
        assert_eq!(
            output.detections.detector_id.as_deref(),
            Some("test-model-adapter")
        );
    }
}
