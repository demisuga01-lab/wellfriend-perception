//! JSON-compatible runtime schema and scalar runtime adapter shared by C and WASM bindings.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use wellfriend_perception_core::{
    BoundaryGeometry, BoundaryKind, BoundaryResult, DetectionGeometry, ImageBuffer, ImageShape,
    PerceptionError, Point2, Quad, Stride,
};
use wellfriend_perception_intelligence::{
    detection::{candidate_quad, manual_quad_candidate},
    domains::document::ClassicalDocumentDetector,
    fusion::QuadFusionEngine,
    quality::ScalarQualityAnalyzer,
    readiness::{CaptureGuidance, CaptureReadiness, CaptureReadinessEngine, CaptureReadinessInput},
    refinement::QuadRefiner,
    temporal::QuadTemporalTracker,
};
use wellfriend_perception_reconstruction::{
    AspectRatioPolicy, CropMarginPolicy, OrientationPolicy, PlanarDocumentInput,
    PlanarDocumentReconstructor, PlanarReconstructionConfig,
};
use wellfriend_perception_restoration::{
    ConditionVector, DeviceClass, DocumentFilterGraph, DocumentFilterPreset,
};

/// Maximum decoded image bytes accepted by this scalar bridge before image allocation.
pub const MAX_RUNTIME_IMAGE_BYTES: usize = 64 * 1024 * 1024;

/// Stable coordinate convention for every runtime geometry payload.
pub const COORDINATE_CONVENTION: &str =
    "origin_top_left;x_right;y_down;unit_source_pixels;quad_tl_tr_br_bl";

/// A point in source-image pixels.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct RuntimePoint2 {
    pub x: f32,
    pub y: f32,
}

impl From<Point2> for RuntimePoint2 {
    fn from(value: Point2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
impl From<RuntimePoint2> for Point2 {
    fn from(value: RuntimePoint2) -> Self {
        Point2::new(value.x, value.y)
    }
}

/// Ordered TL, TR, BR, BL quad used for page reconstruction and manual corrections.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RuntimeQuad {
    pub points: [RuntimePoint2; 4],
}
impl From<Quad> for RuntimeQuad {
    fn from(value: Quad) -> Self {
        Self {
            points: value.points.map(Into::into),
        }
    }
}
impl TryFrom<RuntimeQuad> for Quad {
    type Error = String;
    fn try_from(value: RuntimeQuad) -> Result<Self, Self::Error> {
        let quad = Quad {
            points: value.points.map(Into::into),
        };
        quad.validate().map_err(display_error)?;
        Ok(quad)
    }
}

/// Runtime image materialized in JSON only for scalar integration and test fixtures.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RuntimeImage {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixel_format: String,
    pub bytes: Vec<u8>,
}

/// Boundary geometry that can represent non-quad future SDK results without fabricating data.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", content = "geometry", rename_all = "snake_case")]
pub enum RuntimeBoundaryGeometry {
    Point(RuntimePoint2),
    Line {
        start: RuntimePoint2,
        end: RuntimePoint2,
    },
    Segment {
        start: RuntimePoint2,
        end: RuntimePoint2,
    },
    Quad(RuntimeQuad),
    Polygon(Vec<RuntimePoint2>),
    Circle {
        center: RuntimePoint2,
        radius: f32,
    },
    Ellipse {
        center: RuntimePoint2,
        radius_x: f32,
        radius_y: f32,
        rotation_degrees: f32,
    },
    FreeformContour(Vec<RuntimePoint2>),
    MaskPlaceholder,
    SurfaceOutlinePlaceholder,
    Unknown,
}

/// Serialized evidence and limitations for a boundary result.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RuntimeBoundary {
    pub kind: String,
    pub geometry: Option<RuntimeBoundaryGeometry>,
    pub confidence: f32,
    pub variance: Option<f32>,
    pub edge_support: f32,
    pub source: String,
    pub statuses: Vec<String>,
    pub limitations: Vec<String>,
}

/// Request for scalar frame analysis; all fields are optional so C callers can pass `{}`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct AnalyzeFrameRequest {
    pub frame_index: Option<u64>,
    pub manual_quad: Option<RuntimeQuad>,
}

/// Real scalar quality representation preserved across bindings.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RuntimeQualityMetric {
    pub raw_value: f32,
    pub normalized_score: f32,
    pub confidence: f32,
}

/// Result from a real scalar analysis pass.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AnalyzeFrameResponse {
    pub schema_version: u32,
    pub engine: String,
    pub engine_mode: String,
    pub coordinate_convention: String,
    pub quality: BTreeMap<String, RuntimeQualityMetric>,
    pub candidates: Vec<RuntimeQuad>,
    pub fused_quad: Option<RuntimeQuad>,
    pub refined_quad: Option<RuntimeQuad>,
    pub boundary: RuntimeBoundary,
    pub capture_readiness: String,
    pub capture_readiness_score: f32,
    pub guidance: Vec<String>,
    pub diagnostics: Vec<String>,
    pub timings_micros: BTreeMap<String, u64>,
}

/// Selected-geometry reconstruction request.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReconstructPageRequest {
    pub quad: RuntimeQuad,
    #[serde(default = "default_output_long_edge")]
    pub output_long_edge: u32,
    #[serde(default)]
    pub aspect_policy: Option<String>,
    #[serde(default)]
    pub orientation_policy: Option<String>,
    #[serde(default)]
    pub crop_margin_policy: Option<String>,
}
fn default_output_long_edge() -> u32 {
    1600
}

/// Real scalar planar reconstruction output, including materialized pixels for first bindings.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReconstructPageResponse {
    pub schema_version: u32,
    pub engine: String,
    pub engine_mode: String,
    pub image: RuntimeImage,
    pub confidence: f32,
    pub diagnostics: Vec<String>,
}

/// Implemented document filter request.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ApplyFilterRequest {
    pub preset: String,
}

/// Real scalar restoration output; advanced requests return an explicit deferred diagnostic.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ApplyFilterResponse {
    pub schema_version: u32,
    pub engine: String,
    pub engine_mode: String,
    pub image: RuntimeImage,
    pub applied_processor_ids: Vec<String>,
    pub diagnostics: Vec<String>,
}

/// Structured runtime error response shared by C and WASM adapters.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RuntimeError {
    pub schema_version: u32,
    pub error: RuntimeDiagnostic,
}
/// Runtime failure diagnostic; image bytes are intentionally never included.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RuntimeDiagnostic {
    pub code: String,
    pub message: String,
}

/// Thin runtime engine; algorithms stay in their owning core crates.
#[derive(Clone, Debug, Default)]
pub struct RuntimeEngine;

impl RuntimeEngine {
    /// Parses and bounds the configuration object. MP10 does not expose unchecked tuning knobs.
    pub fn new(config_json: &str) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(config_json)
            .map_err(|error| format!("invalid config JSON: {error}"))?;
        if !value.is_object() {
            return Err("config JSON must be an object".into());
        }
        Ok(Self)
    }

    /// Runs the actual scalar analysis pipeline, preserving failure evidence rather than guessing edges.
    pub fn analyze(
        &self,
        bytes: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        pixel_format: &str,
        request_json: &str,
    ) -> Result<String, String> {
        let request: AnalyzeFrameRequest = parse_request(request_json, "AnalyzeFrameRequest")?;
        let image = decode_image(bytes, width, height, stride, pixel_format)?;
        let start = std::time::Instant::now();
        let quality = ScalarQualityAnalyzer::default()
            .analyze(&image)
            .map_err(display_error)?;
        let quality_elapsed = start.elapsed().as_micros() as u64;
        let detection_start = std::time::Instant::now();
        let mut set = ClassicalDocumentDetector::default()
            .detect_image(&image)
            .map_err(display_error)?
            .detections;
        if let Some(manual) = request.manual_quad {
            set.candidates
                .push(manual_quad_candidate(manual.try_into()?).map_err(display_error)?);
        }
        let detection_elapsed = detection_start.elapsed().as_micros() as u64;
        let fusion_start = std::time::Instant::now();
        let fusion = QuadFusionEngine::default()
            .fuse(&[set.clone()])
            .map_err(display_error)?;
        let refinement = if fusion.fused_geometry.is_some() {
            QuadRefiner::default()
                .refine(&image, &fusion)
                .map_err(display_error)?
        } else {
            Default::default()
        };
        let mut temporal = QuadTemporalTracker::default();
        let temporal_update = temporal
            .update(
                wellfriend_perception_core::FrameIndex(request.frame_index.unwrap_or(0)),
                refined_quad(&refinement).or_else(|| fused_quad(&fusion)),
            )
            .map_err(display_error)?;
        let readiness = CaptureReadinessEngine::default().evaluate(CaptureReadinessInput {
            quality: &quality,
            fusion: &fusion,
            refinement: &refinement,
            temporal: &temporal_update.state,
            image_shape: image.shape(),
        });
        let fusion_elapsed = fusion_start.elapsed().as_micros() as u64;
        let candidate_quads = set
            .candidates
            .iter()
            .filter_map(|candidate| candidate_quad(candidate).ok())
            .map(Into::into)
            .collect::<Vec<_>>();
        let refined = refined_quad(&refinement).map(Into::into);
        let fused = fused_quad(&fusion).map(Into::into);
        let boundary = if let Some(quad) = refined.clone().or(fused.clone()) {
            RuntimeBoundary {
                kind: "quad".into(),
                geometry: Some(RuntimeBoundaryGeometry::Quad(quad)),
                confidence: refinement.confidence.value().max(fusion.confidence.value()),
                variance: None,
                edge_support: fusion.confidence.value(),
                source: "native_scalar:classical_document+fusion+refinement".into(),
                statuses: vec!["visible_edge_found".into()],
                limitations: vec!["scalar baseline; heuristic confidence is not calibrated".into()],
            }
        } else {
            runtime_boundary(&BoundaryResult::insufficient_evidence(
                "native_scalar",
                "no validated visible document boundary; manual correction is required",
            ))
        };
        let quality_metrics = quality
            .metrics
            .iter()
            .map(|(name, value)| {
                (
                    name.clone(),
                    RuntimeQualityMetric {
                        raw_value: value.raw_value,
                        normalized_score: value.normalized_score.value(),
                        confidence: value.confidence.value(),
                    },
                )
            })
            .collect();
        let mut diagnostics = quality.diagnostics.clone();
        diagnostics.extend(fusion.diagnostics.clone());
        diagnostics.extend(refinement.diagnostics.clone());
        diagnostics.extend(readiness.diagnostics.clone());
        if boundary.geometry.is_none() {
            diagnostics.push("insufficient_evidence: no boundary was fabricated".into());
        }
        let response = AnalyzeFrameResponse {
            schema_version: 1,
            engine: "wellfriend-perception".into(),
            engine_mode: "native_scalar".into(),
            coordinate_convention: COORDINATE_CONVENTION.into(),
            quality: quality_metrics,
            candidates: candidate_quads,
            fused_quad: fused,
            refined_quad: refined,
            boundary,
            capture_readiness: format_readiness(readiness.readiness).into(),
            capture_readiness_score: readiness.score.value(),
            guidance: readiness
                .guidance
                .iter()
                .map(|value| format_guidance(*value).into())
                .collect(),
            diagnostics,
            timings_micros: BTreeMap::from([
                ("quality".into(), quality_elapsed),
                ("detection".into(), detection_elapsed),
                (
                    "fusion_refinement_temporal_readiness".into(),
                    fusion_elapsed,
                ),
            ]),
        };
        to_json(&response)
    }

    /// Runs real planar reconstruction after validating a caller-selected quad.
    pub fn reconstruct(
        &self,
        bytes: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        pixel_format: &str,
        request_json: &str,
    ) -> Result<String, String> {
        let request: ReconstructPageRequest =
            parse_request(request_json, "ReconstructPageRequest")?;
        if !(256..=4096).contains(&request.output_long_edge) {
            return Err("output_long_edge must be within 256..=4096".into());
        }
        let image = decode_image(bytes, width, height, stride, pixel_format)?;
        let config = PlanarReconstructionConfig {
            target_long_edge: request.output_long_edge,
            aspect_policy: parse_aspect(request.aspect_policy.as_deref())?,
            orientation_policy: parse_orientation(request.orientation_policy.as_deref())?,
            crop_margin_policy: parse_margin(request.crop_margin_policy.as_deref())?,
            ..Default::default()
        };
        let page = PlanarDocumentReconstructor { config }
            .reconstruct_page(&PlanarDocumentInput {
                image,
                quad: request.quad.try_into()?,
            })
            .map_err(display_error)?;
        to_json(&ReconstructPageResponse {
            schema_version: 1,
            engine: "wellfriend-perception".into(),
            engine_mode: "native_scalar".into(),
            image: runtime_image(&page.image)?,
            confidence: page.confidence.value.value(),
            diagnostics: page.trace.diagnostics,
        })
    }

    /// Applies a named scalar filter plan. Deferred presets are deliberately described in diagnostics.
    pub fn apply_filter(
        &self,
        bytes: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        pixel_format: &str,
        request_json: &str,
    ) -> Result<String, String> {
        let request: ApplyFilterRequest = parse_request(request_json, "ApplyFilterRequest")?;
        let image = decode_image(bytes, width, height, stride, pixel_format)?;
        let preset = parse_filter(&request.preset)?;
        let output = DocumentFilterGraph::default()
            .apply(
                preset,
                &image,
                &ConditionVector::default(),
                DeviceClass::Unknown,
            )
            .map_err(display_error)?;
        to_json(&ApplyFilterResponse {
            schema_version: 1,
            engine: "wellfriend-perception".into(),
            engine_mode: "native_scalar".into(),
            image: runtime_image(&output.image)?,
            applied_processor_ids: output
                .applied_processors
                .into_iter()
                .map(|item| item.as_str().into())
                .collect(),
            diagnostics: output.diagnostics,
        })
    }
}

fn parse_request<T: for<'de> Deserialize<'de>>(json: &str, name: &str) -> Result<T, String> {
    if json.len() > 256 * 1024 {
        return Err(format!("{name} exceeds 262144 bytes"));
    }
    serde_json::from_str(json).map_err(|error| format!("invalid {name}: {error}"))
}
fn to_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| format!("serialization failed: {error}"))
}
fn display_error(error: PerceptionError) -> String {
    error.to_string()
}

fn decode_image(
    bytes: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    pixel_format: &str,
) -> Result<ImageBuffer, String> {
    if bytes.len() > MAX_RUNTIME_IMAGE_BYTES {
        return Err(format!(
            "image exceeds {MAX_RUNTIME_IMAGE_BYTES} byte runtime limit"
        ));
    }
    let format = match pixel_format.to_ascii_lowercase().as_str() {
        "gray8" | "gray_8" => wellfriend_perception_core::PixelFormat::Gray8,
        "rgb8" | "rgb_8" => wellfriend_perception_core::PixelFormat::Rgb8,
        "bgr8" | "bgr_8" => wellfriend_perception_core::PixelFormat::Bgr8,
        "rgba8" | "rgba_8" => wellfriend_perception_core::PixelFormat::Rgba8,
        _ => return Err(format!("unsupported runtime pixel format: {pixel_format}")),
    };
    let shape = ImageShape::new(width, height).map_err(display_error)?;
    ImageBuffer::new_with_stride(shape, format, Stride(stride as usize), bytes.to_vec())
        .map_err(display_error)
}
fn runtime_image(image: &ImageBuffer) -> Result<RuntimeImage, String> {
    Ok(RuntimeImage {
        width: image.width(),
        height: image.height(),
        stride: image
            .stride()
            .0
            .try_into()
            .map_err(|_| "stride exceeds u32".to_string())?,
        pixel_format: image.pixel_format().to_string(),
        bytes: image.as_bytes().to_vec(),
    })
}
fn fused_quad(fusion: &wellfriend_perception_core::FusionResult) -> Option<Quad> {
    match fusion.fused_geometry {
        Some(DetectionGeometry::Quad(value)) => Some(value),
        _ => None,
    }
}
fn refined_quad(refinement: &wellfriend_perception_core::RefinementResult) -> Option<Quad> {
    match refinement.refined_geometry {
        Some(DetectionGeometry::Quad(value)) => Some(value),
        _ => None,
    }
}

fn runtime_boundary(value: &BoundaryResult) -> RuntimeBoundary {
    RuntimeBoundary {
        kind: boundary_kind(value.kind).into(),
        geometry: value.geometry.as_ref().and_then(runtime_geometry),
        confidence: value.confidence.value(),
        variance: value.uncertainty.variance,
        edge_support: value.edge_support.value(),
        source: value.source.clone(),
        statuses: value
            .statuses
            .iter()
            .map(|status| boundary_status(*status).into())
            .collect(),
        limitations: value.limitations.clone(),
    }
}
fn runtime_geometry(value: &BoundaryGeometry) -> Option<RuntimeBoundaryGeometry> {
    Some(match value {
        BoundaryGeometry::Point(point) => RuntimeBoundaryGeometry::Point((*point).into()),
        BoundaryGeometry::Line { start, end } => RuntimeBoundaryGeometry::Line {
            start: (*start).into(),
            end: (*end).into(),
        },
        BoundaryGeometry::Segment { start, end } => RuntimeBoundaryGeometry::Segment {
            start: (*start).into(),
            end: (*end).into(),
        },
        BoundaryGeometry::Quad(quad) => RuntimeBoundaryGeometry::Quad((*quad).into()),
        BoundaryGeometry::Polygon(polygon) => RuntimeBoundaryGeometry::Polygon(
            polygon.points.iter().copied().map(Into::into).collect(),
        ),
        BoundaryGeometry::Circle { center, radius } => RuntimeBoundaryGeometry::Circle {
            center: (*center).into(),
            radius: *radius,
        },
        BoundaryGeometry::Ellipse {
            center,
            radius_x,
            radius_y,
            rotation_degrees,
        } => RuntimeBoundaryGeometry::Ellipse {
            center: (*center).into(),
            radius_x: *radius_x,
            radius_y: *radius_y,
            rotation_degrees: *rotation_degrees,
        },
        BoundaryGeometry::FreeformContour(contour) => RuntimeBoundaryGeometry::FreeformContour(
            contour.points.iter().copied().map(Into::into).collect(),
        ),
        BoundaryGeometry::Mask(_) => RuntimeBoundaryGeometry::MaskPlaceholder,
        BoundaryGeometry::SurfaceOutline(_) => RuntimeBoundaryGeometry::SurfaceOutlinePlaceholder,
        BoundaryGeometry::Unknown => RuntimeBoundaryGeometry::Unknown,
    })
}
fn boundary_kind(value: BoundaryKind) -> &'static str {
    match value {
        BoundaryKind::Point => "point",
        BoundaryKind::Line => "line",
        BoundaryKind::Segment => "segment",
        BoundaryKind::Quad => "quad",
        BoundaryKind::Polygon => "polygon",
        BoundaryKind::Circle => "circle",
        BoundaryKind::Ellipse => "ellipse",
        BoundaryKind::FreeformContour => "freeform_contour",
        BoundaryKind::Mask => "mask",
        BoundaryKind::SurfaceOutline => "surface_outline",
        BoundaryKind::Unknown => "unknown",
    }
}
fn boundary_status(value: wellfriend_perception_core::BoundaryDiagnosticStatus) -> &'static str {
    match value {
        wellfriend_perception_core::BoundaryDiagnosticStatus::VisibleEdgeFound => {
            "visible_edge_found"
        }
        wellfriend_perception_core::BoundaryDiagnosticStatus::WeakEdge => "weak_edge",
        wellfriend_perception_core::BoundaryDiagnosticStatus::AmbiguousBoundary => {
            "ambiguous_boundary"
        }
        wellfriend_perception_core::BoundaryDiagnosticStatus::OccludedBoundary => {
            "occluded_boundary"
        }
        wellfriend_perception_core::BoundaryDiagnosticStatus::SaturatedBoundary => {
            "saturated_boundary"
        }
        wellfriend_perception_core::BoundaryDiagnosticStatus::LowContrastBoundary => {
            "low_contrast_boundary"
        }
        wellfriend_perception_core::BoundaryDiagnosticStatus::OutOfFrame => "out_of_frame",
        wellfriend_perception_core::BoundaryDiagnosticStatus::InsufficientEvidence => {
            "insufficient_evidence"
        }
        wellfriend_perception_core::BoundaryDiagnosticStatus::ManualRequired => "manual_required",
    }
}
fn format_guidance(value: CaptureGuidance) -> &'static str {
    match value {
        CaptureGuidance::NoDocument => "NO_DOCUMENT",
        CaptureGuidance::DocumentCutOff => "DOCUMENT_CUT_OFF",
        CaptureGuidance::MoveCloser => "MOVE_CLOSER",
        CaptureGuidance::MoveFarther => "MOVE_FARTHER",
        CaptureGuidance::HoldSteady => "HOLD_STEADY",
        CaptureGuidance::TooBlurry => "TOO_BLURRY",
        CaptureGuidance::TooDark => "TOO_DARK",
        CaptureGuidance::TooBright => "TOO_BRIGHT",
        CaptureGuidance::GlareDetected => "GLARE_DETECTED",
        CaptureGuidance::LowConfidence => "LOW_CONFIDENCE",
        CaptureGuidance::LowDetectorAgreement => "LOW_DETECTOR_AGREEMENT",
        CaptureGuidance::Ready => "READY",
    }
}
fn format_readiness(value: CaptureReadiness) -> &'static str {
    match value {
        CaptureReadiness::NotReady => "NOT_READY",
        CaptureReadiness::AlmostReady => "ALMOST_READY",
        CaptureReadiness::Ready => "READY",
        CaptureReadiness::CaptureNow => "CAPTURE_NOW",
    }
}
fn parse_aspect(value: Option<&str>) -> Result<AspectRatioPolicy, String> {
    match value.unwrap_or("free_from_quad") {
        "free_from_quad" => Ok(AspectRatioPolicy::FreeFromQuad),
        _ => Err("unsupported aspect_policy for MP10 runtime".into()),
    }
}
fn parse_orientation(value: Option<&str>) -> Result<OrientationPolicy, String> {
    match value.unwrap_or("preserve_source") {
        "preserve_source" => Ok(OrientationPolicy::PreserveSource),
        "long_edge_vertical" => Ok(OrientationPolicy::LongEdgeVertical),
        "long_edge_horizontal" => Ok(OrientationPolicy::LongEdgeHorizontal),
        _ => Err("unsupported orientation_policy for MP10 runtime".into()),
    }
}
fn parse_margin(value: Option<&str>) -> Result<CropMarginPolicy, String> {
    match value.unwrap_or("safe_inner") {
        "none" => Ok(CropMarginPolicy::None),
        "safe_inner" => Ok(CropMarginPolicy::SafeInner),
        "include_border" => Ok(CropMarginPolicy::IncludeBorder),
        _ => Err("unsupported crop_margin_policy for MP10 runtime".into()),
    }
}
fn parse_filter(value: &str) -> Result<DocumentFilterPreset, String> {
    match value {
        "Original" | "original" => Ok(DocumentFilterPreset::Original),
        "Auto" | "auto" => Ok(DocumentFilterPreset::Auto),
        "Clean" | "clean" => Ok(DocumentFilterPreset::Clean),
        "Color" | "color" => Ok(DocumentFilterPreset::Color),
        "Grayscale" | "grayscale" => Ok(DocumentFilterPreset::Grayscale),
        "B&W" | "black_and_white" | "black-and-white" => Ok(DocumentFilterPreset::BlackAndWhite),
        "Receipt" | "receipt" => Ok(DocumentFilterPreset::Receipt),
        "Book" | "book" => Ok(DocumentFilterPreset::Book),
        "Whiteboard" | "whiteboard" => Ok(DocumentFilterPreset::Whiteboard),
        "PhotoDocument" | "photo_document" => Ok(DocumentFilterPreset::PhotoDocument),
        _ => Err(format!("unsupported filter preset: {value}")),
    }
}

/// Creates an intentionally small structured error JSON document.
pub fn runtime_error_json(code: &str, message: &str) -> String {
    serde_json::to_string(&RuntimeError {
        schema_version: 1,
        error: RuntimeDiagnostic {
            code: code.into(),
            message: message.into(),
        },
    })
    .unwrap_or_else(|_| {
        "{\"schema_version\":1,\"error\":{\"code\":\"serialization_error\"}}".into()
    })
}

#[cfg(test)]
mod tests {
    use super::{
        AnalyzeFrameResponse, RuntimeBoundaryGeometry, RuntimeEngine, RuntimePoint2, RuntimeQuad,
    };

    fn centered_page() -> Vec<u8> {
        let mut image = vec![20; 64 * 48];
        for y in 8..40 {
            for x in 12..52 {
                image[y * 64 + x] = 230;
            }
        }
        image
    }
    #[test]
    fn scalar_runtime_returns_structured_no_fabrication_response() {
        let json = RuntimeEngine::new("{}")
            .unwrap()
            .analyze(&centered_page(), 64, 48, 64, "Gray8", "{}")
            .unwrap();
        let response: AnalyzeFrameResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.schema_version, 1);
        assert!(!response.boundary.source.is_empty());
    }
    #[test]
    fn manual_quad_roundtrips_through_the_real_pipeline() {
        let request = serde_json::json!({ "manual_quad": RuntimeQuad { points: [RuntimePoint2 { x: 12.0, y: 8.0 }, RuntimePoint2 { x: 51.0, y: 8.0 }, RuntimePoint2 { x: 51.0, y: 39.0 }, RuntimePoint2 { x: 12.0, y: 39.0 }] } });
        let json = RuntimeEngine::new("{}")
            .unwrap()
            .analyze(&centered_page(), 64, 48, 64, "gray8", &request.to_string())
            .unwrap();
        let response: AnalyzeFrameResponse = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            response.boundary.geometry,
            Some(RuntimeBoundaryGeometry::Quad(_))
        ));
    }
}
