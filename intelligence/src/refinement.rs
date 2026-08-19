//! Scalar edge-supported quad refinement after fusion consensus.

use wellfriend_perception_core::{
    Confidence, DetectionGeometry, DetectionSet, DetectionSource, FusionResult, PerceptionError,
    PerceptionResult, Point2, Quad, RefinementResult,
    geometry::{RansacConfig, line_intersection, ransac_line_fit},
};
use wellfriend_perception_image::{gradient_magnitude, grayscale};

use crate::detection::document_quad_candidate;

/// Configuration for deterministic high-resolution edge and corner refinement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuadRefinementConfig {
    /// Samples evaluated on each coarse edge.
    pub samples_per_edge: usize,
    /// Search radius along each coarse edge normal in pixels.
    pub normal_search_radius: i32,
    /// Minimum selected gradient magnitude for an accepted edge point.
    pub minimum_gradient: f32,
    /// RANSAC inlier threshold for an edge fit.
    pub line_inlier_threshold: f32,
    /// Maximum mean corner movement accepted from the coarse geometry.
    pub maximum_delta_pixels: f32,
}

impl Default for QuadRefinementConfig {
    fn default() -> Self {
        Self {
            samples_per_edge: 32,
            normal_search_radius: 5,
            minimum_gradient: 18.0,
            line_inlier_threshold: 1.5,
            maximum_delta_pixels: 24.0,
        }
    }
}

/// Scalar reference refiner that converts gradient maxima into fitted edge lines.
#[derive(Clone, Debug, Default)]
pub struct QuadRefiner {
    /// Public refinement thresholds.
    pub config: QuadRefinementConfig,
}

impl QuadRefiner {
    /// Refines a fused quad and returns a safe low-confidence fallback on weak edges.
    pub fn refine(
        &self,
        image: &wellfriend_perception_core::ImageBuffer,
        fused: &FusionResult,
    ) -> PerceptionResult<RefinementResult> {
        let coarse = match &fused.fused_geometry {
            Some(DetectionGeometry::Quad(quad)) => *quad,
            _ => {
                return Err(PerceptionError::UnsupportedOperation {
                    operation: "quad refinement requires fused quad geometry",
                });
            }
        };
        self.refine_quad(image, coarse, fused.confidence)
    }

    /// Refines a supplied coarse quad directly for detector and test pipelines.
    pub fn refine_quad(
        &self,
        image: &wellfriend_perception_core::ImageBuffer,
        coarse: Quad,
        prior_confidence: Confidence,
    ) -> PerceptionResult<RefinementResult> {
        coarse.validate()?;
        let gray = grayscale(image)?;
        let gradients = gradient_magnitude(&gray)?;
        let mut lines = Vec::new();
        let mut accepted = 0usize;
        for edge in coarse.edges() {
            let points = edge_gradient_maxima(
                &gradients,
                image.width(),
                image.height(),
                edge.start,
                edge.end,
                self.config,
            );
            if points.len() >= 4 {
                if let Ok(fit) = ransac_line_fit(
                    &points,
                    RansacConfig {
                        inlier_threshold: self.config.line_inlier_threshold,
                        iterations: 96,
                        ..RansacConfig::default()
                    },
                ) {
                    lines.push(fit.fit.line);
                    accepted += points.len();
                    continue;
                }
            }
            lines.push(wellfriend_perception_core::Line2 {
                a: edge.start,
                b: edge.end,
            });
        }
        let refined = Quad {
            points: [
                line_intersection(lines[3], lines[0]),
                line_intersection(lines[0], lines[1]),
                line_intersection(lines[1], lines[2]),
                line_intersection(lines[2], lines[3]),
            ]
            .map(|point| point.unwrap_or_default()),
        };
        let usable = refined.validate().is_ok();
        let delta = if usable {
            mean_corner_delta(coarse, refined)
        } else {
            0.0
        };
        let accepted_refinement =
            usable && delta <= self.config.maximum_delta_pixels && accepted >= 16;
        let output = if accepted_refinement { refined } else { coarse };
        let evidence =
            (accepted as f32 / (self.config.samples_per_edge * 4) as f32).clamp(0.0, 1.0);
        let confidence = if accepted_refinement {
            Confidence::new((0.45 * prior_confidence.value() + 0.55 * evidence).clamp(0.0, 1.0))?
        } else {
            Confidence::new((prior_confidence.value() * 0.55).clamp(0.0, 1.0))?
        };
        let mut candidate = document_quad_candidate(
            DetectionSource::External("quad-refiner-v1".into()),
            output,
            confidence.value(),
            "quad-refiner-v1",
        )?;
        candidate
            .attributes
            .insert("refinement_delta".into(), delta.to_string());
        candidate
            .attributes
            .insert("edge_samples".into(), accepted.to_string());
        Ok(RefinementResult {
            candidates: DetectionSet {
                candidates: vec![candidate],
                detector_id: Some("quad-refiner-v1".into()),
                diagnostics: vec![if accepted_refinement {
                    "refined with edge-normal gradient maxima and RANSAC line intersections".into()
                } else {
                    "weak or inconsistent edge evidence; returned validated coarse geometry".into()
                }],
            },
            refined_geometry: Some(DetectionGeometry::Quad(output)),
            refinement_delta: if accepted_refinement { delta } else { 0.0 },
            confidence,
            diagnostics: Vec::new(),
        })
    }
}

fn edge_gradient_maxima(
    gradients: &[f32],
    width: u32,
    height: u32,
    start: Point2,
    end: Point2,
    config: QuadRefinementConfig,
) -> Vec<Point2> {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = dx.hypot(dy);
    if length <= f32::EPSILON {
        return Vec::new();
    }
    let normal = Point2::new(-dy / length, dx / length);
    let mut points = Vec::new();
    for index in 1..config.samples_per_edge.saturating_sub(1) {
        let t = index as f32 / (config.samples_per_edge - 1) as f32;
        let center = Point2::new(start.x + dx * t, start.y + dy * t);
        let mut best = (0.0, center);
        for offset in -config.normal_search_radius..=config.normal_search_radius {
            let point = Point2::new(
                center.x + normal.x * offset as f32,
                center.y + normal.y * offset as f32,
            );
            let x = point.x.round().clamp(0.0, (width - 1) as f32) as usize;
            let y = point.y.round().clamp(0.0, (height - 1) as f32) as usize;
            let value = gradients[y * width as usize + x];
            if value > best.0 {
                best = (value, point);
            }
        }
        if best.0 >= config.minimum_gradient {
            points.push(best.1);
        }
    }
    points
}

fn mean_corner_delta(left: Quad, right: Quad) -> f32 {
    left.points
        .iter()
        .zip(right.points)
        .map(|(a, b)| a.distance(b))
        .sum::<f32>()
        / 4.0
}
