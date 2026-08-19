//! Quality checks that occur after geometric resampling.

use wellfriend_perception_core::{PerceptionResult, Quad, QualityReport, Score};
use wellfriend_perception_image::grayscale;
use wellfriend_perception_intelligence::quality::ScalarQualityAnalyzer;

use crate::{PageQualityAfterReconstruction, ReconstructionQuality};

/// Full post-reconstruction report kept separate from pre-capture quality evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ReconstructionQualityReport {
    /// Reuses the generic quality metrics rather than reimplementing them.
    pub output_quality: QualityReport,
    /// Canonical reconstruction-only risk summary.
    pub reconstruction: ReconstructionQuality,
    /// Fraction of output allocated to intentional margins.
    pub margin_coverage_loss: Score,
    /// Risk that page-border content is clipped by selected policy.
    pub border_clipping_risk: Score,
    /// Additional implementation diagnostics.
    pub diagnostics: Vec<String>,
}

/// Evaluates scalar quality and conservative transform risks after a planar warp.
pub fn evaluate_reconstruction_quality(
    image: &wellfriend_perception_core::ImageBuffer,
    source_quad: Quad,
    output_width: u32,
    output_height: u32,
    margins: (u32, u32, u32, u32),
    output_aspect: f32,
) -> PerceptionResult<PageQualityAfterReconstruction> {
    let report = evaluate_reconstruction_quality_report(
        image,
        source_quad,
        output_width,
        output_height,
        margins,
        output_aspect,
    )?;
    Ok(PageQualityAfterReconstruction {
        output_blur_score: metric_score(&report.output_quality, "blur_laplacian_variance"),
        output_exposure_score: metric_score(&report.output_quality, "mean_luminance"),
        output_contrast_score: metric_score(&report.output_quality, "contrast_percentile_range"),
        border_clipping_risk: report.border_clipping_risk,
        summary: report.reconstruction,
    })
}

/// Produces the complete post-reconstruction report for condition routing and benchmarks.
pub fn evaluate_reconstruction_quality_report(
    image: &wellfriend_perception_core::ImageBuffer,
    source_quad: Quad,
    output_width: u32,
    output_height: u32,
    margins: (u32, u32, u32, u32),
    output_aspect: f32,
) -> PerceptionResult<ReconstructionQualityReport> {
    let grayscale = grayscale(image)?;
    let output_quality = ScalarQualityAnalyzer::default().analyze(&grayscale)?;
    let top = source_quad.points[0].distance(source_quad.points[1]);
    let right = source_quad.points[1].distance(source_quad.points[2]);
    let bottom = source_quad.points[2].distance(source_quad.points[3]);
    let left = source_quad.points[3].distance(source_quad.points[0]);
    let horizontal_ratio = top.max(bottom) / top.min(bottom).max(f32::EPSILON);
    let vertical_ratio = left.max(right) / left.min(right).max(f32::EPSILON);
    let warp_stretch = ((horizontal_ratio.max(vertical_ratio) - 1.0) / 3.0).clamp(0.0, 1.0);
    let inferred_aspect = ((top + bottom) * 0.5) / ((left + right) * 0.5).max(f32::EPSILON);
    let aspect_distortion =
        ((inferred_aspect.ln() - output_aspect.max(f32::EPSILON).ln()).abs() / 1.2).clamp(0.0, 1.0);
    let margin_pixels = (margins.0 + margins.2) as u64 * output_height as u64
        + (margins.1 + margins.3) as u64 * output_width as u64;
    let output_pixels = output_width as u64 * output_height as u64;
    let coverage_loss =
        (margin_pixels.min(output_pixels) as f32 / output_pixels.max(1) as f32).clamp(0.0, 1.0);
    let border_clipping = if margins == (0, 0, 0, 0) { 0.35 } else { 0.08 };
    let output_quality_score = [
        metric_score(&output_quality, "blur_laplacian_variance").value(),
        metric_score(&output_quality, "mean_luminance").value(),
        metric_score(&output_quality, "contrast_percentile_range").value(),
    ]
    .into_iter()
    .sum::<f32>()
        / 3.0;
    Ok(ReconstructionQualityReport {
        output_quality,
        reconstruction: ReconstructionQuality {
            output_quality_score: Score::new(output_quality_score)?,
            coverage_loss: Score::new(coverage_loss)?,
            warp_stretch_risk: Score::new(warp_stretch)?,
            aspect_distortion_risk: Score::new(aspect_distortion)?,
            diagnostics: vec![
                "post-reconstruction scalar quality uses MP3 quality analyzer".into(),
                "warp stretch and aspect distortion are conservative geometry heuristics".into(),
            ],
        },
        margin_coverage_loss: Score::new(coverage_loss)?,
        border_clipping_risk: Score::new(border_clipping)?,
        diagnostics: vec![
            "border clipping risk is a policy diagnostic, not pixel-perfect content analysis"
                .into(),
        ],
    })
}

fn metric_score(report: &QualityReport, name: &str) -> Score {
    report
        .metrics
        .get(name)
        .map(|metric| metric.normalized_score)
        .unwrap_or_default()
}
