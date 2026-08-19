//! Deterministic quad consensus, provenance retention, and manual override policy.

use wellfriend_perception_core::{
    Confidence, DetectionGeometry, DetectionSet, DetectionSource, FusionResult, PerceptionError,
    PerceptionResult, Point2, Polygon, Quad, Score,
};

use crate::detection::{candidate_quad, document_quad_candidate};

/// Tunables for conservative quad fusion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuadFusionConfig {
    /// Maximum normalized mean corner distance for candidates to form a consensus.
    pub maximum_corner_distance: f32,
    /// Minimum composite agreement to contribute to the fused candidate.
    pub minimum_agreement: f32,
    /// Manual candidates above this source rule override automatic consensus.
    pub manual_override_enabled: bool,
}

impl Default for QuadFusionConfig {
    fn default() -> Self {
        Self {
            maximum_corner_distance: 0.12,
            minimum_agreement: 0.48,
            manual_override_enabled: true,
        }
    }
}

/// Fusion engine for independently attributable quad candidates.
#[derive(Clone, Debug, Default)]
pub struct QuadFusionEngine {
    /// Public deterministic consensus settings.
    pub config: QuadFusionConfig,
}

impl QuadFusionEngine {
    /// Fuses candidates from independent detector outputs without discarding provenance.
    pub fn fuse(&self, sets: &[DetectionSet]) -> PerceptionResult<FusionResult> {
        let mut valid = Vec::new();
        let mut rejected = Vec::new();
        for set in sets {
            for candidate in &set.candidates {
                match candidate_quad(candidate) {
                    Ok(quad) => valid.push((candidate, quad)),
                    Err(_) => rejected.push(candidate.source.clone()),
                }
            }
        }
        if valid.is_empty() {
            return Ok(FusionResult {
                rejected_sources: rejected,
                diagnostics: vec!["no valid quad candidates available for fusion".into()],
                ..Default::default()
            });
        }
        let manual: Vec<_> = valid
            .iter()
            .filter(|(candidate, _)| candidate.source == DetectionSource::Manual)
            .collect();
        if self.config.manual_override_enabled && !manual.is_empty() {
            let (candidate, quad) = *manual
                .iter()
                .max_by(|(left, _), (right, _)| left.score.value().total_cmp(&right.score.value()))
                .ok_or(PerceptionError::InsufficientPoints {
                    required: 1,
                    actual: 0,
                })?;
            let mut fused = (*candidate).clone();
            fused
                .attributes
                .insert("fusion_policy".into(), "manual_override".into());
            return Ok(FusionResult {
                candidates: DetectionSet {
                    candidates: vec![fused.clone()],
                    detector_id: Some("quad-fusion-v1".into()),
                    diagnostics: vec![
                        "manual geometry override accepted after quad validation".into(),
                    ],
                },
                fused_geometry: Some(DetectionGeometry::Quad(*quad)),
                confidence: Confidence::new(1.0)?,
                contributing_sources: vec![DetectionSource::Manual],
                rejected_sources: valid
                    .iter()
                    .filter(|(other, _)| other.source != DetectionSource::Manual)
                    .map(|(other, _)| other.source.clone())
                    .chain(rejected)
                    .collect(),
                disagreement_score: Score::new(0.0)?,
                diagnostics: vec![
                    "manual candidate has explicit high reliability but remains validated".into(),
                ],
            });
        }
        valid.sort_by(|(left, _), (right, _)| {
            candidate_weight(right).total_cmp(&candidate_weight(left))
        });
        let (_, anchor) = valid[0];
        let mut contributors = Vec::new();
        let mut disagreements = Vec::new();
        for (candidate, quad) in &valid {
            let agreement = quad_agreement(anchor, *quad);
            if agreement >= self.config.minimum_agreement
                && normalized_corner_distance(anchor, *quad) <= self.config.maximum_corner_distance
            {
                contributors.push((*candidate, *quad));
                disagreements.push(1.0 - agreement);
            } else {
                rejected.push(candidate.source.clone());
            }
        }
        if contributors.is_empty() {
            return Err(PerceptionError::DegenerateGeometry {
                reason: "fusion consensus unexpectedly discarded the anchor".into(),
            });
        }
        let fused_quad = weighted_quad(&contributors)?;
        let confidence = contributors
            .iter()
            .map(|(candidate, _)| candidate_weight(candidate))
            .sum::<f32>()
            / contributors.len() as f32;
        let disagreement = if disagreements.is_empty() {
            1.0
        } else {
            disagreements.iter().sum::<f32>() / disagreements.len() as f32
        };
        let mut fused_candidate = document_quad_candidate(
            DetectionSource::External("quad-fusion-v1".into()),
            fused_quad,
            confidence,
            "quad-fusion-v1",
        )?;
        fused_candidate
            .attributes
            .insert("fusion_policy".into(), "weighted_consensus".into());
        fused_candidate
            .attributes
            .insert("contributor_count".into(), contributors.len().to_string());
        Ok(FusionResult {
            candidates: DetectionSet {
                candidates: vec![fused_candidate],
                detector_id: Some("quad-fusion-v1".into()),
                diagnostics: vec!["candidate grouping uses corner, overlap, orientation, area, and center agreement".into()],
            },
            fused_geometry: Some(DetectionGeometry::Quad(fused_quad)),
            confidence: Confidence::new(confidence.clamp(0.0, 1.0))?,
            contributing_sources: contributors
                .iter()
                .map(|(candidate, _)| candidate.source.clone())
                .collect(),
            rejected_sources: rejected,
            disagreement_score: Score::new(disagreement.clamp(0.0, 1.0))?,
            diagnostics: vec![format!("{} candidates contributed", contributors.len())],
        })
    }
}

/// Convex-quad IoU using Sutherland-Hodgman clipping.
pub fn quad_iou(left: Quad, right: Quad) -> f32 {
    let subject = left.polygon().points;
    let clip = right.polygon().points;
    let intersection = clip_convex_polygon(subject, &clip);
    let intersection_area = Polygon {
        points: intersection,
    }
    .area();
    let union = left.polygon().area() + right.polygon().area() - intersection_area;
    if union <= f32::EPSILON {
        0.0
    } else {
        (intersection_area / union).clamp(0.0, 1.0)
    }
}

/// Composite geometry agreement across corners, overlap, edge orientation, area, and center.
pub fn quad_agreement(left: Quad, right: Quad) -> f32 {
    let corner = 1.0 - normalized_corner_distance(left, right).clamp(0.0, 1.0);
    let overlap = quad_iou(left, right);
    let area = {
        let larger = left.polygon().area().max(right.polygon().area()).max(1.0);
        1.0 - (left.polygon().area() - right.polygon().area()).abs() / larger
    };
    let center = match (left.polygon().centroid(), right.polygon().centroid()) {
        (Ok(a), Ok(b)) => {
            let scale = diagonal(left).max(diagonal(right)).max(1.0);
            1.0 - (a.distance(b) / scale).clamp(0.0, 1.0)
        }
        _ => 0.0,
    };
    let orientation = edge_orientation_agreement(left, right);
    (0.34 * corner + 0.28 * overlap + 0.14 * area + 0.14 * center + 0.10 * orientation)
        .clamp(0.0, 1.0)
}

fn candidate_weight(candidate: &wellfriend_perception_core::DetectionCandidate) -> f32 {
    source_reliability(&candidate.source)
        * candidate.confidence.score.value()
        * candidate.score.value()
}

fn source_reliability(source: &DetectionSource) -> f32 {
    match source {
        DetectionSource::Manual => 1.0,
        DetectionSource::Ml => 0.90,
        DetectionSource::Classical => 0.75,
        DetectionSource::Temporal => 0.60,
        DetectionSource::External(_) => 0.70,
    }
}

fn weighted_quad(
    candidates: &[(&wellfriend_perception_core::DetectionCandidate, Quad)],
) -> PerceptionResult<Quad> {
    let total = candidates
        .iter()
        .map(|(candidate, _)| candidate_weight(candidate))
        .sum::<f32>();
    if total <= f32::EPSILON {
        return Err(PerceptionError::DegenerateGeometry {
            reason: "fusion candidates have zero effective weight".into(),
        });
    }
    let mut points = [Point2::default(); 4];
    for (candidate, quad) in candidates {
        let weight = candidate_weight(candidate) / total;
        for (target, source) in points.iter_mut().zip(quad.points) {
            target.x += source.x * weight;
            target.y += source.y * weight;
        }
    }
    let quad = Quad { points };
    quad.validate()?;
    Ok(quad)
}

fn normalized_corner_distance(left: Quad, right: Quad) -> f32 {
    let scale = diagonal(left).max(diagonal(right)).max(1.0);
    let mut best = f32::INFINITY;
    for reversed in [false, true] {
        for shift in 0..4 {
            let distance = (0..4)
                .map(|index| {
                    let mapped = if reversed {
                        right.points[(4 + shift - index) % 4]
                    } else {
                        right.points[(index + shift) % 4]
                    };
                    left.points[index].distance(mapped)
                })
                .sum::<f32>()
                / 4.0;
            best = best.min(distance);
        }
    }
    best / scale
}

fn diagonal(quad: Quad) -> f32 {
    quad.points[0]
        .distance(quad.points[2])
        .max(quad.points[1].distance(quad.points[3]))
}

fn edge_orientation_agreement(left: Quad, right: Quad) -> f32 {
    left.edges()
        .iter()
        .zip(right.edges())
        .map(|(a, b)| {
            let ax = a.end.x - a.start.x;
            let ay = a.end.y - a.start.y;
            let bx = b.end.x - b.start.x;
            let by = b.end.y - b.start.y;
            ((ax * bx + ay * by) / (ax.hypot(ay) * bx.hypot(by)).max(1e-4))
                .abs()
                .clamp(0.0, 1.0)
        })
        .sum::<f32>()
        / 4.0
}

fn clip_convex_polygon(mut subject: Vec<Point2>, clip: &[Point2]) -> Vec<Point2> {
    let winding = Polygon {
        points: clip.to_vec(),
    }
    .signed_area()
    .signum();
    for (start, end) in clip
        .iter()
        .zip(clip.iter().cycle().skip(1))
        .take(clip.len())
    {
        let input = subject;
        subject = Vec::new();
        if input.is_empty() {
            break;
        }
        let mut previous = *input.last().unwrap_or(start);
        for current in input {
            let current_inside = inside_edge(current, *start, *end, winding);
            let previous_inside = inside_edge(previous, *start, *end, winding);
            if current_inside != previous_inside {
                if let Some(point) = segment_line_intersection(previous, current, *start, *end) {
                    subject.push(point);
                }
            }
            if current_inside {
                subject.push(current);
            }
            previous = current;
        }
    }
    subject
}

fn inside_edge(point: Point2, start: Point2, end: Point2, winding: f32) -> bool {
    let cross = (end.x - start.x) * (point.y - start.y) - (end.y - start.y) * (point.x - start.x);
    cross * winding >= -1e-4
}

fn segment_line_intersection(a: Point2, b: Point2, c: Point2, d: Point2) -> Option<Point2> {
    wellfriend_perception_core::geometry::line_intersection(
        wellfriend_perception_core::Line2 { a, b },
        wellfriend_perception_core::Line2 { a: c, b: d },
    )
}
