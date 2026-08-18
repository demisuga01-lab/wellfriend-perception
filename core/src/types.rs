use std::collections::BTreeMap;

/// Stable identifier for an observation supplied to the pipeline.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObservationId(pub String);

/// Origin of an observation. New variants can be represented by `External`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservationSource {
    Camera,
    File,
    Video,
    Sensor,
    Network,
    External(String),
}

/// Extensible metadata attached to an observation without leaking domain fields into core.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObservationMetadata {
    pub timestamp_ms: Option<i64>,
    pub source: Option<ObservationSource>,
    pub attributes: BTreeMap<String, String>,
}

/// Pixel layouts supported by baseline image buffers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Gray8,
    Rgb8,
    Rgba8,
}

impl PixelFormat {
    pub const fn channels(self) -> usize {
        match self {
            Self::Gray8 => 1,
            Self::Rgb8 => 3,
            Self::Rgba8 => 4,
        }
    }
}

/// Owned, interleaved image pixels. The constructor validates dimensions and stride.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub data: Vec<u8>,
}

impl ImageBuffer {
    pub fn new(
        width: u32,
        height: u32,
        pixel_format: PixelFormat,
        data: Vec<u8>,
    ) -> Result<Self, String> {
        let expected = width as usize * height as usize * pixel_format.channels();
        if data.len() != expected {
            return Err(format!(
                "pixel buffer has {} bytes; expected {expected}",
                data.len()
            ));
        }
        Ok(Self {
            width,
            height,
            pixel_format,
            data,
        })
    }
}

/// Non-owning full-image view used by processing interfaces.
#[derive(Clone, Copy, Debug)]
pub struct ImageView<'a> {
    pub width: u32,
    pub height: u32,
    pub pixel_format: PixelFormat,
    pub data: &'a [u8],
}

impl<'a> From<&'a ImageBuffer> for ImageView<'a> {
    fn from(value: &'a ImageBuffer) -> Self {
        Self {
            width: value.width,
            height: value.height,
            pixel_format: value.pixel_format,
            data: &value.data,
        }
    }
}

/// Non-owning tensor description for model-runtime adapters.
#[derive(Clone, Copy, Debug)]
pub struct TensorView<'a> {
    pub shape: &'a [usize],
    pub values: &'a [f32],
}

/// A single sensor observation and its optional frame payload.
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub id: ObservationId,
    pub metadata: ObservationMetadata,
    pub image: Option<ImageBuffer>,
}

/// Time-ordered collection of observations, such as a camera stream segment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObservationFrame {
    pub observations: Vec<Observation>,
}

/// Standard quality metrics shared by all domain packs.
#[derive(Clone, Debug, PartialEq)]
pub enum QualityMetric {
    Blur,
    Noise,
    Exposure,
    Saturation,
    Contrast,
    Motion,
    Glare,
    Occlusion,
    Confidence,
    DomainSpecific(String),
}

/// Named normalized quality values; directionality is defined by the analyzer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QualityVector(pub BTreeMap<String, f32>);

/// Measured quality plus detector diagnostics.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QualityReport {
    pub vector: QualityVector,
    pub diagnostics: Vec<String>,
}

/// A two-dimensional point in a declared coordinate system.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}
/// A three-dimensional point in a declared coordinate system.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
/// Infinite 2D line represented by two points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line2 {
    pub a: Point2,
    pub b: Point2,
}
/// Finite 2D line segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment2 {
    pub start: Point2,
    pub end: Point2,
}
/// Ordered 2D polygon boundary.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polygon {
    pub points: Vec<Point2>,
}
/// Four-corner polygon with stable corner order defined by its producer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub points: [Point2; 4],
}
/// Axis-aligned 2D bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
/// A binary image mask; its coordinate mapping belongs to the associated result.
#[derive(Clone, Debug, PartialEq)]
pub struct Mask {
    pub width: u32,
    pub height: u32,
    pub values: Vec<u8>,
}
/// Generic sampled or parametric surface representation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Surface {
    pub vertices: Vec<Point3>,
    pub indices: Vec<u32>,
}
/// Position and orientation expressed by the declaring domain pack.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub translation: Point3,
    pub rotation_xyzw: [f32; 4],
}
/// Homogeneous 2D transform in row-major order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub matrix: [[f32; 3]; 3],
}
/// Homogeneous 3D transform in row-major order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform3D {
    pub matrix: [[f32; 4]; 4],
}
/// Dense mapping from output pixels to source coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct DenseWarpField {
    pub width: u32,
    pub height: u32,
    pub vectors: Vec<Point2>,
}

/// How a detector obtained a candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DetectionSource {
    Classical,
    Ml,
    Temporal,
    Manual,
    External(String),
}
/// Calibrated confidence with an explicit interval when available.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetectionConfidence {
    pub score: f32,
    pub lower: f32,
    pub upper: f32,
}
/// Qualitative and numerical uncertainty information.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Uncertainty {
    pub covariance: Vec<f32>,
    pub notes: Vec<String>,
}
/// Domain-neutral candidate payload; interpretation is declared by `kind` and the domain pack.
#[derive(Clone, Debug, PartialEq)]
pub struct DetectionCandidate {
    pub kind: String,
    pub source: DetectionSource,
    pub confidence: DetectionConfidence,
    pub geometry: Option<Polygon>,
    pub uncertainty: Uncertainty,
    pub attributes: BTreeMap<String, String>,
}
/// Candidates produced for one observation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DetectionSet {
    pub candidates: Vec<DetectionCandidate>,
    pub diagnostics: Vec<String>,
}

/// Fusion output with traceable contributing candidates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FusionResult {
    pub candidates: DetectionSet,
    pub diagnostics: Vec<String>,
}
/// Refinement output for one selected target.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RefinementResult {
    pub candidates: DetectionSet,
    pub diagnostics: Vec<String>,
}
/// Persistent temporal state represented without imposing a tracker implementation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TemporalState {
    pub stable: bool,
    pub confidence: f32,
    pub diagnostics: Vec<String>,
}

/// Domain-selected mathematical reconstruction model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeometryModel {
    Planar,
    Surface,
    Volumetric,
    Geospatial,
    Photogrammetric,
    Custom(String),
}
/// Reconstruction product and its optional geometry artifacts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReconstructionResult {
    pub transform_2d: Option<Transform2D>,
    pub transform_3d: Option<Transform3D>,
    pub surface: Option<Surface>,
    pub diagnostics: Vec<String>,
}

/// Condition scores used to route specialized processors.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConditionVector(pub BTreeMap<String, f32>);
/// Ordered processor identifiers selected by the router.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessingPlan {
    pub processor_ids: Vec<String>,
    pub diagnostics: Vec<String>,
}
/// Standardized processor result for graph execution and observability.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessorResult {
    pub output: Option<ImageBuffer>,
    pub confidence: f32,
    pub diagnostics: Vec<String>,
}

/// Declared operating envelope used by the specialist router for plan selection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessorCapabilities {
    pub capabilities: Vec<String>,
    pub expected_benefit: f32,
    pub estimated_cost_ms: u32,
    pub supported_device_classes: Vec<String>,
    pub confidence: f32,
    pub diagnostics: Vec<String>,
}

/// A typed semantic region, independent of OCR or document-specific labels.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticRegion {
    pub kind: String,
    pub geometry: Option<Polygon>,
    pub confidence: f32,
    pub attributes: BTreeMap<String, String>,
}
/// Semantic regions and relationships emitted by a semantic engine.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticResult {
    pub regions: Vec<SemanticRegion>,
    pub relationships: Vec<(usize, usize, String)>,
}
/// Exportable, domain-owned structured output.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StructuredOutput {
    pub schema: String,
    pub payload: String,
    pub diagnostics: Vec<String>,
}

/// Immutable per-run context passed across pipeline stages.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PipelineContext {
    pub run_id: String,
    pub domain: String,
    pub attributes: BTreeMap<String, String>,
}
/// Ordered pipeline stages. Packs may intentionally omit stages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineStage {
    Input,
    Quality,
    Detection,
    Fusion,
    Refinement,
    Temporal,
    Reconstruction,
    Condition,
    Routing,
    Restoration,
    Semantics,
    Export,
}
