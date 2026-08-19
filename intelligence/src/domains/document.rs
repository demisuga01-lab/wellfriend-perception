//! Classical document-quad reference implementation and document quality extensions.
//!
//! The detector is intentionally scalar and deterministic. It uses bright connected
//! components plus boundary/line fitting, so it is a useful baseline and not a
//! substitute for a trained segmentation or corner-regression model.

use wellfriend_perception_core::{
    Confidence, DetectionSet, DetectionSource, DetectorCapabilities, ImageBuffer, PerceptionResult,
    Point2, Quad, QualityMeasurement, QualityReport, Score,
    geometry::{RansacConfig, line_intersection, ransac_line_fit},
};
use wellfriend_perception_image::{
    gaussian_blur_gray, gradient_magnitude, grayscale, otsu_threshold, resize_bilinear,
};

use crate::{
    detection::{DetectorInput, DetectorOutput, PerceptionDetector, document_quad_candidate},
    quality::ScalarQualityAnalyzer,
};

/// Configuration for the MP3 classical bright-page document detector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClassicalDocumentDetectorConfig {
    /// Longest working side; larger inputs are downscaled deterministically.
    pub working_max_dimension: u32,
    /// Minimum connected component coverage to consider as a page.
    pub minimum_area_ratio: f32,
    /// Maximum candidates returned after scoring.
    pub max_candidates: usize,
    /// Boundary-point distance used for robust edge line fitting.
    pub line_support_distance: f32,
}

impl Default for ClassicalDocumentDetectorConfig {
    fn default() -> Self {
        Self {
            working_max_dimension: 640,
            minimum_area_ratio: 0.05,
            max_candidates: 4,
            line_support_distance: 2.5,
        }
    }
}

/// Deterministic scalar detector for bright, bounded document-like quadrilaterals.
#[derive(Clone, Debug, Default)]
pub struct ClassicalDocumentDetector {
    /// Public detector tuning parameters.
    pub config: ClassicalDocumentDetectorConfig,
}

/// Explainable components used to form a document candidate score.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DocumentCandidateScore {
    /// Quad area divided by image area.
    pub area_ratio: f32,
    /// Angle and edge-length plausibility.
    pub aspect_plausibility: f32,
    /// One for validated convex quads, zero otherwise.
    pub convexity: f32,
    /// Gradient support sampled along candidate edges.
    pub edge_support: f32,
    /// Local gradient energy at corners.
    pub corner_strength: f32,
    /// Bright/dark separation around the component boundary.
    pub border_contrast: f32,
    /// Area ratio to bounding box.
    pub rectangularity: f32,
    /// Coverage preference that rejects tiny components.
    pub image_coverage: f32,
    /// Single-frame baseline; temporal modules update this later.
    pub orientation_stability: f32,
    /// Weighted heuristic raw score.
    pub raw_score: f32,
}

impl ClassicalDocumentDetector {
    /// Detects document candidates without a centered-rectangle fallback.
    pub fn detect_image(&self, image: &ImageBuffer) -> PerceptionResult<DetectorOutput> {
        let gray = grayscale(image)?;
        let (working, scale_x, scale_y) = working_image(&gray, self.config.working_max_dimension)?;
        let smoothed = gaussian_blur_gray(&working)?;
        let gradients = gradient_magnitude(&smoothed)?;
        let edge_threshold = gradient_threshold(&gradients);
        let edge_map: Vec<bool> = gradients
            .iter()
            .map(|value| *value >= edge_threshold)
            .collect();
        let closed_edges = close_binary(&edge_map, working.width(), working.height());
        let foreground_threshold = otsu_threshold(&smoothed)?;
        let foreground = threshold_foreground(&smoothed, foreground_threshold)?;
        let components = connected_components(&foreground, working.width(), working.height());
        let mut candidates = Vec::new();

        for component in components {
            let coverage =
                component.pixels.len() as f32 / (working.width() * working.height()) as f32;
            if coverage < self.config.minimum_area_ratio {
                continue;
            }
            let boundary = component_boundary(
                &foreground,
                working.width(),
                working.height(),
                &component.pixels,
            );
            let component_luma = component_mean_luma(&smoothed, &component.pixels)?;
            let Some(coarse) = quad_from_extrema(&boundary) else {
                continue;
            };
            let quad = fit_component_lines(&boundary, coarse, self.config.line_support_distance)
                .unwrap_or(coarse);
            let original_quad = scale_quad(quad, scale_x, scale_y);
            if original_quad.validate().is_err() {
                continue;
            }
            let score = score_document_quad(
                original_quad,
                image.width(),
                image.height(),
                &gradients,
                working.width(),
                working.height(),
                scale_x,
                scale_y,
                component_luma,
                foreground_threshold,
            )?;
            if score.raw_score >= 0.15 {
                let mut candidate = document_quad_candidate(
                    DetectionSource::Classical,
                    original_quad,
                    score.raw_score,
                    "document-classical-v1",
                )?;
                candidate
                    .attributes
                    .insert("area_ratio".into(), score.area_ratio.to_string());
                candidate
                    .attributes
                    .insert("edge_support".into(), score.edge_support.to_string());
                candidate
                    .attributes
                    .insert("heuristic_score".into(), score.raw_score.to_string());
                candidate.diagnostics.messages.extend([
                    "pipeline: grayscale -> resize -> blur -> gradient -> edge threshold -> morphology -> components -> boundary lines -> quad".into(),
                    format!("edge pixels after close: {}", closed_edges.iter().filter(|value| **value).count()),
                ]);
                candidates.push(candidate);
            }
        }
        candidates.sort_by(|left, right| right.score.value().total_cmp(&left.score.value()));
        candidates.truncate(self.config.max_candidates);
        Ok(DetectorOutput {
            detections: DetectionSet {
                candidates,
                detector_id: Some(self.id().into()),
                diagnostics: vec![
                    "classical detector uses bright-component boundary approximation; dark-page-on-light-background is deferred".into(),
                    format!("working edge threshold: {edge_threshold:.3}"),
                ],
            },
            capabilities: self.capabilities(),
            diagnostics: Vec::new(),
        })
    }
}

impl PerceptionDetector for ClassicalDocumentDetector {
    fn id(&self) -> &str {
        "document-classical-v1"
    }

    fn capabilities(&self) -> DetectorCapabilities {
        DetectorCapabilities {
            geometry_kinds: vec!["quad".into()],
            model_backed: false,
            accepts_manual_geometry: false,
            supported_runtime_classes: vec!["scalar-cpu".into()],
        }
    }

    fn detect(&self, input: DetectorInput<'_>) -> PerceptionResult<DetectorOutput> {
        self.detect_image(input.image)
    }
}

/// Document-scoped extensions derived from generic quality and a detected quad.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DocumentQualityExtensions {
    /// Fraction of the image covered by a valid page candidate.
    pub page_coverage: f32,
    /// Fraction of page corners inside the frame.
    pub page_visibility: f32,
    /// Mean normalized distance of corners from frame edges.
    pub border_visibility: f32,
    /// Dark-pixel baseline within the page bounds; a coarse shadow proxy only.
    pub shadow_likelihood_baseline: f32,
    /// Explicit placeholder until surface geometry is available.
    pub curvature_likelihood_placeholder: f32,
    /// Explicit placeholder until a planar fit residual is computed.
    pub residual_perspective_placeholder: f32,
}

/// Adds document guidance without placing document assumptions in the generic analyzer.
pub fn apply_document_quality_extensions(
    report: &mut QualityReport,
    image: &ImageBuffer,
    quad: Option<Quad>,
) -> PerceptionResult<DocumentQualityExtensions> {
    let Some(quad) = quad else {
        report.warnings.push("document_cut_off".into());
        report.recommended_actions.push("no_document".into());
        return Ok(DocumentQualityExtensions {
            page_coverage: 0.0,
            page_visibility: 0.0,
            border_visibility: 0.0,
            shadow_likelihood_baseline: 0.0,
            curvature_likelihood_placeholder: 0.0,
            residual_perspective_placeholder: 0.0,
        });
    };
    quad.validate()?;
    let image_area = (image.width() * image.height()) as f32;
    let coverage = (quad.polygon().area() / image_area).clamp(0.0, 1.0);
    let visibility = quad
        .points
        .iter()
        .filter(|point| {
            point.x >= 0.0
                && point.y >= 0.0
                && point.x < image.width() as f32
                && point.y < image.height() as f32
        })
        .count() as f32
        / 4.0;
    let smallest_side = image.width().min(image.height()) as f32;
    let border_visibility = quad
        .points
        .iter()
        .map(|point| {
            point
                .x
                .min(point.y)
                .min((image.width() as f32 - 1.0 - point.x).max(0.0))
                .min((image.height() as f32 - 1.0 - point.y).max(0.0))
                / (smallest_side * 0.08).max(1.0)
        })
        .map(|value| value.clamp(0.0, 1.0))
        .sum::<f32>()
        / 4.0;
    let gray = wellfriend_perception_image::grayscale(image)?;
    let bounds = quad.bounding_box()?;
    let left = bounds.x.max(0.0) as u32;
    let top = bounds.y.max(0.0) as u32;
    let right = (bounds.x + bounds.width).min(image.width() as f32) as u32;
    let bottom = (bounds.y + bounds.height).min(image.height() as f32) as u32;
    let mut dark = 0usize;
    let mut total = 0usize;
    for y in top..bottom {
        for x in left..right {
            if gray.get_u8(x, y, 0)? < 55 {
                dark += 1;
            }
            total += 1;
        }
    }
    let shadow = if total == 0 {
        0.0
    } else {
        dark as f32 / total as f32
    };
    add_extension(report, "document_page_coverage", coverage, coverage, 0.85)?;
    add_extension(
        report,
        "document_page_visibility",
        visibility,
        visibility,
        0.85,
    )?;
    add_extension(
        report,
        "document_border_visibility",
        border_visibility,
        border_visibility,
        0.7,
    )?;
    add_extension(
        report,
        "document_shadow_likelihood_baseline",
        shadow,
        1.0 - shadow,
        0.35,
    )?;
    add_extension(
        report,
        "document_curvature_likelihood_placeholder",
        0.0,
        0.5,
        0.0,
    )?;
    add_extension(
        report,
        "document_residual_perspective_placeholder",
        0.0,
        0.5,
        0.0,
    )?;
    if coverage < 0.10 || visibility < 1.0 || border_visibility < 0.05 {
        report.warnings.push("document_cut_off".into());
        report.recommended_actions.push("move_farther".into());
    } else if coverage < 0.25 {
        report.recommended_actions.push("move_closer".into());
    }
    Ok(DocumentQualityExtensions {
        page_coverage: coverage,
        page_visibility: visibility,
        border_visibility,
        shadow_likelihood_baseline: shadow,
        curvature_likelihood_placeholder: 0.0,
        residual_perspective_placeholder: 0.0,
    })
}

/// Convenience document analysis that keeps generic and document quality distinct.
pub fn analyze_document_quality(
    image: &ImageBuffer,
    quad: Option<Quad>,
) -> PerceptionResult<(QualityReport, DocumentQualityExtensions)> {
    let mut report = ScalarQualityAnalyzer::default().analyze(image)?;
    let extensions = apply_document_quality_extensions(&mut report, image, quad)?;
    Ok((report, extensions))
}

#[derive(Clone, Debug)]
struct Component {
    pixels: Vec<(u32, u32)>,
}

fn working_image(image: &ImageBuffer, maximum: u32) -> PerceptionResult<(ImageBuffer, f32, f32)> {
    let longest = image.width().max(image.height());
    if longest <= maximum {
        return Ok((image.clone(), 1.0, 1.0));
    }
    let scale = maximum as f32 / longest as f32;
    let width = (image.width() as f32 * scale).round().max(1.0) as u32;
    let height = (image.height() as f32 * scale).round().max(1.0) as u32;
    Ok((
        resize_bilinear(image, width, height)?,
        image.width() as f32 / width as f32,
        image.height() as f32 / height as f32,
    ))
}

fn threshold_foreground(image: &ImageBuffer, threshold: u8) -> PerceptionResult<Vec<bool>> {
    let mut minimum = u8::MAX;
    let mut maximum = 0u8;
    for y in 0..image.height() {
        for value in image.view().row(y)? {
            minimum = minimum.min(*value);
            maximum = maximum.max(*value);
        }
    }
    if minimum == maximum {
        return Ok(vec![
            false;
            image.width() as usize * image.height() as usize
        ]);
    }
    let mut output = Vec::with_capacity(image.width() as usize * image.height() as usize);
    for y in 0..image.height() {
        for value in image.view().row(y)? {
            output.push(*value > threshold);
        }
    }
    Ok(close_binary(&output, image.width(), image.height()))
}

fn gradient_threshold(gradients: &[f32]) -> f32 {
    let mut sorted = gradients.to_vec();
    sorted.sort_by(f32::total_cmp);
    sorted[(sorted.len() * 3 / 4).min(sorted.len().saturating_sub(1))].max(18.0)
}

fn close_binary(input: &[bool], width: u32, height: u32) -> Vec<bool> {
    erode_binary(&dilate_binary(input, width, height), width, height)
}

fn dilate_binary(input: &[bool], width: u32, height: u32) -> Vec<bool> {
    let mut output = vec![false; input.len()];
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            output[y as usize * width as usize + x as usize] = (-1..=1)
                .any(|dy| (-1..=1).any(|dx| sample_mask(input, width, height, x + dx, y + dy)));
        }
    }
    output
}

fn erode_binary(input: &[bool], width: u32, height: u32) -> Vec<bool> {
    let mut output = vec![false; input.len()];
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            output[y as usize * width as usize + x as usize] = (-1..=1)
                .all(|dy| (-1..=1).all(|dx| sample_mask(input, width, height, x + dx, y + dy)));
        }
    }
    output
}

fn sample_mask(mask: &[bool], width: u32, height: u32, x: i32, y: i32) -> bool {
    x >= 0
        && y >= 0
        && x < width as i32
        && y < height as i32
        && mask[y as usize * width as usize + x as usize]
}

fn connected_components(mask: &[bool], width: u32, height: u32) -> Vec<Component> {
    let mut visited = vec![false; mask.len()];
    let mut components = Vec::new();
    for start in 0..mask.len() {
        if visited[start] || !mask[start] {
            continue;
        }
        let mut pixels = Vec::new();
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(index) = stack.pop() {
            let x = (index % width as usize) as i32;
            let y = (index / width as usize) as i32;
            pixels.push((x as u32, y as u32));
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && ny >= 0 && nx < width as i32 && ny < height as i32 {
                    let neighbor = ny as usize * width as usize + nx as usize;
                    if mask[neighbor] && !visited[neighbor] {
                        visited[neighbor] = true;
                        stack.push(neighbor);
                    }
                }
            }
        }
        components.push(Component { pixels });
    }
    components
}

fn component_mean_luma(image: &ImageBuffer, pixels: &[(u32, u32)]) -> PerceptionResult<f32> {
    if pixels.is_empty() {
        return Ok(0.0);
    }
    Ok(pixels
        .iter()
        .map(|(x, y)| image.get_u8(*x, *y, 0).map(f32::from))
        .collect::<PerceptionResult<Vec<_>>>()?
        .iter()
        .sum::<f32>()
        / pixels.len() as f32)
}

fn component_boundary(
    mask: &[bool],
    width: u32,
    height: u32,
    component: &[(u32, u32)],
) -> Vec<Point2> {
    component
        .iter()
        .filter(|(x, y)| {
            [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|(dx, dy)| !sample_mask(mask, width, height, *x as i32 + dx, *y as i32 + dy))
        })
        .map(|(x, y)| Point2::new(*x as f32, *y as f32))
        .collect()
}

fn quad_from_extrema(points: &[Point2]) -> Option<Quad> {
    if points.len() < 4 {
        return None;
    }
    let select = |score: fn(Point2) -> f32, descending: bool| {
        points.iter().copied().reduce(|best, point| {
            if (score(point) > score(best)) == descending {
                point
            } else {
                best
            }
        })
    };
    let quad = Quad {
        points: [
            select(|p| p.x + p.y, false)?,
            select(|p| p.x - p.y, true)?,
            select(|p| p.x + p.y, true)?,
            select(|p| p.x - p.y, false)?,
        ],
    };
    quad.validate().ok().map(|_| quad)
}

fn fit_component_lines(points: &[Point2], coarse: Quad, support_distance: f32) -> Option<Quad> {
    let mut lines = Vec::new();
    for edge in coarse.edges() {
        let supported: Vec<_> = points
            .iter()
            .copied()
            .filter(|point| distance_to_segment(*point, edge.start, edge.end) <= support_distance)
            .collect();
        if supported.len() < 4 {
            return None;
        }
        let fit = ransac_line_fit(
            &supported,
            RansacConfig {
                inlier_threshold: support_distance,
                iterations: 64,
                ..RansacConfig::default()
            },
        )
        .ok()?;
        lines.push(fit.fit.line);
    }
    let quad = Quad {
        points: [
            line_intersection(lines[3], lines[0])?,
            line_intersection(lines[0], lines[1])?,
            line_intersection(lines[1], lines[2])?,
            line_intersection(lines[2], lines[3])?,
        ],
    };
    quad.validate().ok().map(|_| quad)
}

fn scale_quad(quad: Quad, scale_x: f32, scale_y: f32) -> Quad {
    Quad {
        points: quad
            .points
            .map(|point| Point2::new(point.x * scale_x, point.y * scale_y)),
    }
}

#[allow(clippy::too_many_arguments)]
fn score_document_quad(
    quad: Quad,
    image_width: u32,
    image_height: u32,
    gradients: &[f32],
    working_width: u32,
    working_height: u32,
    scale_x: f32,
    scale_y: f32,
    mean_luma: f32,
    threshold: u8,
) -> PerceptionResult<DocumentCandidateScore> {
    quad.validate()?;
    let image_area = (image_width as f32 * image_height as f32).max(1.0);
    let area_ratio = (quad.polygon().area() / image_area).clamp(0.0, 1.0);
    let bounds = quad.bounding_box()?;
    let rectangularity =
        (quad.polygon().area() / (bounds.width * bounds.height).max(1.0)).clamp(0.0, 1.0);
    let edges = quad.edges();
    let angle_quality = edges
        .iter()
        .zip(edges.iter().cycle().skip(1))
        .map(|(first, second)| {
            let ax = first.end.x - first.start.x;
            let ay = first.end.y - first.start.y;
            let bx = second.end.x - second.start.x;
            let by = second.end.y - second.start.y;
            let cosine = (ax * bx + ay * by) / (ax.hypot(ay) * bx.hypot(by)).max(1e-4);
            1.0 - cosine.abs().clamp(0.0, 1.0)
        })
        .sum::<f32>()
        / 4.0;
    let edge_support = edges
        .iter()
        .map(|edge| {
            let samples = 20;
            (0..samples)
                .map(|index| {
                    let t = index as f32 / (samples - 1) as f32;
                    let x = ((edge.start.x + (edge.end.x - edge.start.x) * t) / scale_x)
                        .round()
                        .clamp(0.0, (working_width - 1) as f32)
                        as usize;
                    let y = ((edge.start.y + (edge.end.y - edge.start.y) * t) / scale_y)
                        .round()
                        .clamp(0.0, (working_height - 1) as f32)
                        as usize;
                    gradients[y * working_width as usize + x]
                })
                .filter(|value| *value > 24.0)
                .count() as f32
                / samples as f32
        })
        .sum::<f32>()
        / 4.0;
    let corner_strength = quad
        .points
        .iter()
        .map(|point| {
            let x = (point.x / scale_x)
                .round()
                .clamp(0.0, (working_width - 1) as f32) as usize;
            let y = (point.y / scale_y)
                .round()
                .clamp(0.0, (working_height - 1) as f32) as usize;
            (gradients[y * working_width as usize + x] / 128.0).clamp(0.0, 1.0)
        })
        .sum::<f32>()
        / 4.0;
    let coverage = ((area_ratio - 0.03) / 0.45).clamp(0.0, 1.0);
    let contrast = ((mean_luma - f32::from(threshold)) / 96.0).clamp(0.0, 1.0);
    let raw_score = (0.24 * coverage
        + 0.18 * angle_quality
        + 0.16 * edge_support
        + 0.12 * corner_strength
        + 0.12 * rectangularity
        + 0.10 * contrast
        + 0.08)
        .clamp(0.0, 1.0);
    Ok(DocumentCandidateScore {
        area_ratio,
        aspect_plausibility: angle_quality,
        convexity: 1.0,
        edge_support,
        corner_strength,
        border_contrast: contrast,
        rectangularity,
        image_coverage: coverage,
        orientation_stability: 0.5,
        raw_score,
    })
}

fn distance_to_segment(point: Point2, start: Point2, end: Point2) -> f32 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let denominator = dx * dx + dy * dy;
    if denominator <= f32::EPSILON {
        return point.distance(start);
    }
    let t = (((point.x - start.x) * dx + (point.y - start.y) * dy) / denominator).clamp(0.0, 1.0);
    point.distance(Point2::new(start.x + t * dx, start.y + t * dy))
}

fn add_extension(
    report: &mut QualityReport,
    name: &str,
    raw: f32,
    score: f32,
    confidence: f32,
) -> PerceptionResult<()> {
    let score = score.clamp(0.0, 1.0);
    report.vector.0.insert(name.into(), score);
    report.metrics.insert(
        name.into(),
        QualityMeasurement {
            raw_value: raw,
            normalized_score: Score::new(score)?,
            confidence: Confidence::new(confidence)?,
            diagnostics: if confidence == 0.0 {
                vec!["placeholder".into()]
            } else {
                Vec::new()
            },
        },
    );
    Ok(())
}
