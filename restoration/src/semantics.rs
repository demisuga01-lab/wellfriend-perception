//! Stable semantic and structured-output seams before OCR and exporters arrive.

use std::collections::BTreeMap;

use wellfriend_perception_core::{Confidence, Polygon};

/// Semantic confidence remains distinct from a geometry score.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SemanticConfidence {
    /// Bounded confidence in the semantic interpretation.
    pub value: Confidence,
}

/// Generic object types that work across perception domains.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticObjectKind {
    /// Text or text-like mark.
    Text,
    /// Table-like structure.
    Table,
    /// Figure or illustration.
    Figure,
    /// Signed region.
    Signature,
    /// Stamp or seal region.
    Stamp,
    /// Generic physical object.
    Object,
    /// Surface or material region.
    Surface,
    /// Anomaly region.
    Anomaly,
    /// Measured region.
    Measurement,
    /// Generic region.
    Region,
    /// Domain-defined extension.
    DomainDefined(String),
}

/// One semantic object with optional geometry and extensional metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticObject {
    /// Object kind.
    pub kind: SemanticObjectKind,
    /// Optional polygon in the canonical coordinate system.
    pub region: Option<Polygon>,
    /// Semantic confidence.
    pub confidence: SemanticConfidence,
    /// Domain-owned attributes.
    pub attributes: BTreeMap<String, String>,
}

/// Semantic layer grouping objects that share an interpretation source.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticLayer {
    /// Stable layer identifier.
    pub id: String,
    /// Objects in this layer.
    pub objects: Vec<SemanticObject>,
}

/// Directed relationship between semantic object indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticRelationship {
    /// Source object index.
    pub from: usize,
    /// Target object index.
    pub to: usize,
    /// Stable relationship name such as `contains` or `reads_before`.
    pub kind: String,
}

/// Typed physical or derived measurement.
#[derive(Clone, Debug, PartialEq)]
pub struct Measurement {
    /// Stable measurement name.
    pub name: String,
    /// Numeric value in declared units.
    pub value: f32,
    /// Unit token.
    pub unit: String,
    /// Bounded confidence in the measurement.
    pub confidence: Confidence,
}

/// Export-ready structured contract without binding to a PDF or OCR implementation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StructuredOutputContract {
    /// Stable schema identifier.
    pub schema: String,
    /// Semantic layers.
    pub layers: Vec<SemanticLayer>,
    /// Directed object relationships.
    pub relationships: Vec<SemanticRelationship>,
    /// Measurements associated with the canonical representation.
    pub measurements: Vec<Measurement>,
    /// Implementation diagnostics.
    pub diagnostics: Vec<String>,
}

/// Required behavior for a future exporter.
pub trait ExporterContract {
    /// Stable output schema produced by this exporter.
    fn schema(&self) -> &str;
    /// Returns true when the exporter accepts this contract without missing required layers.
    fn validate(&self, output: &StructuredOutputContract) -> Result<(), String>;
}

/// Document semantic regions reserved for later OCR/layout integrations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentRegionKind {
    /// Region representing text; no recognition is implied.
    TextRegionPlaceholder,
    /// Region representing a table; no structure extraction is implied.
    TableRegionPlaceholder,
    /// Region representing a figure.
    FigureRegionPlaceholder,
    /// Region representing a signature.
    SignatureRegionPlaceholder,
    /// Region representing a stamp.
    StampRegionPlaceholder,
    /// Region representing a photograph.
    PhotoRegionPlaceholder,
    /// Future reading-order relation.
    ReadingOrderPlaceholder,
}
