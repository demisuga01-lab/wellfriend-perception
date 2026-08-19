//! Shape-general boundary contracts.
//!
//! A boundary is evidence, not a promise that hidden visual information has been
//! recovered.  Domain packs may return a quad for a page today and polygons,
//! circles, contours, masks, or surface outlines later.  Consumers must inspect
//! confidence, uncertainty, and diagnostics before automation.

use crate::{BoundingBox, Confidence, Mask, Point2, Polygon, Quad, Surface, Uncertainty};

/// The representational family returned by a boundary detector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryKind {
    /// A single point.
    Point,
    /// An infinite straight line.
    Line,
    /// A finite line segment.
    Segment,
    /// A four-corner polygon, conventionally TL, TR, BR, BL when page-like.
    Quad,
    /// An ordered polygonal outline.
    Polygon,
    /// A circle defined by centre and radius.
    Circle,
    /// An ellipse defined by centre, axes, and rotation.
    Ellipse,
    /// A freeform visible contour. It must not imply occluded parts are known.
    FreeformContour,
    /// A raster boundary or foreground mask.
    Mask,
    /// A projected outline of a surface.
    SurfaceOutline,
    /// The requested boundary cannot yet be classified.
    Unknown,
}

/// Intent used to select a boundary algorithm without embedding a document-only assumption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryDetectionMode {
    /// Page-like quadrilateral detection.
    DocumentPage,
    /// General object outline.
    ObjectOutline,
    /// Parametric geometric shape detection.
    GeometricShape,
    /// Lines, diagrams, or line art.
    LineArt,
    /// General image edges with no domain promise.
    ArbitraryEdge,
    /// Runtime selects a supported mode conservatively.
    Auto,
}

/// Evidence state that must remain visible to products and users.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryDiagnosticStatus {
    /// Strong visible edge evidence supported the result.
    VisibleEdgeFound,
    /// Edge evidence exists but is weak.
    WeakEdge,
    /// Several boundaries are plausible from the observed pixels.
    AmbiguousBoundary,
    /// The desired boundary is covered by another object.
    OccludedBoundary,
    /// Saturation prevented a reliable edge measurement.
    SaturatedBoundary,
    /// The edge has insufficient local contrast.
    LowContrastBoundary,
    /// Part of the needed boundary lies outside the frame.
    OutOfFrame,
    /// Pixels do not support a reliable estimate.
    InsufficientEvidence,
    /// Validated manual geometry is required before reconstructing automatically.
    ManualRequired,
}

/// Shape-specific payload for a boundary estimate.
#[derive(Clone, Debug, PartialEq)]
pub enum BoundaryGeometry {
    /// Point evidence.
    Point(Point2),
    /// Two ordered points defining an infinite line.
    Line { start: Point2, end: Point2 },
    /// Two ordered endpoints.
    Segment { start: Point2, end: Point2 },
    /// Ordered page/object corners.
    Quad(Quad),
    /// Polygonal outline with documented winding.
    Polygon(Polygon),
    /// Circle centre and radius in source-image pixels.
    Circle { center: Point2, radius: f32 },
    /// Ellipse centre, radii, and clockwise degrees in source-image pixels.
    Ellipse {
        center: Point2,
        radius_x: f32,
        radius_y: f32,
        rotation_degrees: f32,
    },
    /// Only observed contour points; absence never implies an invisible continuation.
    FreeformContour(Polygon),
    /// Raster boundary evidence.
    Mask(Mask),
    /// Surface outline evidence.
    SurfaceOutline(Surface),
    /// No representable geometry is available.
    Unknown,
}

/// Explainable result of a domain-neutral boundary operation.
#[derive(Clone, Debug, PartialEq)]
pub struct BoundaryResult {
    /// Shape family chosen by the detector.
    pub kind: BoundaryKind,
    /// Geometry, if evidence supports a representable result.
    pub geometry: Option<BoundaryGeometry>,
    /// Bounded estimate reliability; it is not a claim of ground truth.
    pub confidence: Confidence,
    /// Optional variance/covariance information for future fitting implementations.
    pub uncertainty: Uncertainty,
    /// Bounded score for measured image-edge support.
    pub edge_support: Confidence,
    /// Detector, user, or external provenance identifier.
    pub source: String,
    /// Stable evidence states such as `insufficient_evidence`.
    pub statuses: Vec<BoundaryDiagnosticStatus>,
    /// Human-readable limitations that products must preserve.
    pub limitations: Vec<String>,
}

impl BoundaryResult {
    /// Creates an explicit insufficient-evidence result instead of inventing geometry.
    pub fn insufficient_evidence(source: impl Into<String>, limitation: impl Into<String>) -> Self {
        Self {
            kind: BoundaryKind::Unknown,
            geometry: None,
            confidence: Confidence::default(),
            uncertainty: Uncertainty::default(),
            edge_support: Confidence::default(),
            source: source.into(),
            statuses: vec![
                BoundaryDiagnosticStatus::InsufficientEvidence,
                BoundaryDiagnosticStatus::ManualRequired,
            ],
            limitations: vec![limitation.into()],
        }
    }

    /// Returns an axis-aligned visible bounds convenience for supported shapes.
    pub fn visible_bounds(&self) -> Option<BoundingBox> {
        match self.geometry.as_ref()? {
            BoundaryGeometry::Quad(quad) => quad.polygon().bounding_box().ok(),
            BoundaryGeometry::Polygon(polygon) | BoundaryGeometry::FreeformContour(polygon) => {
                polygon.bounding_box().ok()
            }
            BoundaryGeometry::Circle { center, radius } => Some(BoundingBox {
                x: center.x - radius,
                y: center.y - radius,
                width: radius * 2.0,
                height: radius * 2.0,
            }),
            BoundaryGeometry::Ellipse {
                center,
                radius_x,
                radius_y,
                ..
            } => Some(BoundingBox {
                x: center.x - radius_x,
                y: center.y - radius_y,
                width: radius_x * 2.0,
                height: radius_y * 2.0,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BoundaryDiagnosticStatus, BoundaryGeometry, BoundaryKind, BoundaryResult};
    use crate::{Confidence, Point2, Uncertainty};

    #[test]
    fn non_quad_boundaries_remain_representable() {
        let boundary = BoundaryResult {
            kind: BoundaryKind::Circle,
            geometry: Some(BoundaryGeometry::Circle {
                center: Point2::new(20.0, 15.0),
                radius: 5.0,
            }),
            confidence: Confidence::new(0.7).unwrap(),
            uncertainty: Uncertainty::with_variance(1.0).unwrap(),
            edge_support: Confidence::new(0.8).unwrap(),
            source: "shape-test".into(),
            statuses: vec![BoundaryDiagnosticStatus::VisibleEdgeFound],
            limitations: vec!["visible arc only".into()],
        };
        assert_eq!(boundary.visible_bounds().unwrap().width, 10.0);
    }

    #[test]
    fn insufficient_evidence_never_fabricates_geometry() {
        let boundary = BoundaryResult::insufficient_evidence("scalar", "white-on-white edge");
        assert!(boundary.geometry.is_none());
        assert!(
            boundary
                .statuses
                .contains(&BoundaryDiagnosticStatus::ManualRequired)
        );
    }
}
