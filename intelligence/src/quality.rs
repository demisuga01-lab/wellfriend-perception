//! Conservative scalar quality metrics and machine-readable capture guidance.

use std::collections::BTreeMap;

use wellfriend_perception_core::{
    Confidence, ImageBuffer, PerceptionError, PerceptionResult, QualityMeasurement, QualityReport,
    QualityVector, Score,
    math::{percentile, standard_deviation, variance},
};
use wellfriend_perception_image::{gradient_magnitude, grayscale, laplacian};

/// Tunables for the scalar reference quality analyzer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScalarQualityConfig {
    /// Laplacian variance treated as a reasonably sharp reference image.
    pub sharp_laplacian_reference: f32,
    /// Sobel energy treated as a reasonably textured reference image.
    pub sharp_gradient_reference: f32,
    /// Gray sample below which a pixel is considered underexposed.
    pub underexposed_threshold: u8,
    /// Gray sample above which a pixel is considered overexposed or clipped.
    pub overexposed_threshold: u8,
    /// Bright, low-texture samples contributing to conservative glare likelihood.
    pub glare_gradient_threshold: f32,
}

impl Default for ScalarQualityConfig {
    fn default() -> Self {
        Self {
            sharp_laplacian_reference: 400.0,
            sharp_gradient_reference: 2500.0,
            underexposed_threshold: 30,
            overexposed_threshold: 225,
            glare_gradient_threshold: 12.0,
        }
    }
}

/// Dependency-free quality analyzer for checked image buffers.
#[derive(Clone, Debug, Default)]
pub struct ScalarQualityAnalyzer {
    /// Public scalar thresholds for reproducible tuning.
    pub config: ScalarQualityConfig,
}

impl ScalarQualityAnalyzer {
    /// Measures generic image quality signals. Scores are higher-is-better except
    /// `glare_likelihood`, `underexposed_fraction`, and `overexposed_fraction`.
    pub fn analyze(&self, image: &ImageBuffer) -> PerceptionResult<QualityReport> {
        let gray = grayscale(image)?;
        let samples = packed_gray(&gray)?;
        if samples.is_empty() {
            return Err(PerceptionError::InvalidBuffer {
                expected: 1,
                actual: 0,
            });
        }
        let values: Vec<f32> = samples.iter().map(|value| f32::from(*value)).collect();
        let laplacian_values = laplacian(&gray)?;
        let laplacian_variance = variance(&laplacian_values)?;
        let gradients = gradient_magnitude(&gray)?;
        let gradient_energy = gradients.iter().map(|value| value * value).sum::<f32>()
            / gradients.len().max(1) as f32;
        let mean_luminance = values.iter().sum::<f32>() / values.len() as f32;
        let underexposed = fraction(&samples, |value| {
            *value <= self.config.underexposed_threshold
        });
        let overexposed = fraction(&samples, |value| {
            *value >= self.config.overexposed_threshold
        });
        let dynamic_range = percentile(&values, 0.95)? - percentile(&values, 0.05)?;
        let luminance_stddev = standard_deviation(&values)?;
        let clipped = fraction(&samples, |value| *value == 0 || *value == u8::MAX);
        let blur_score =
            normalized_reference(laplacian_variance, self.config.sharp_laplacian_reference);
        let tenengrad_score =
            normalized_reference(gradient_energy, self.config.sharp_gradient_reference);
        let contrast_score = (dynamic_range / 128.0).clamp(0.0, 1.0);
        let blurred = wellfriend_perception_image::box_blur_gray(&gray)?;
        let blur_samples = packed_gray(&blurred)?;
        let noise_residual = samples
            .iter()
            .zip(blur_samples)
            .map(|(source, smooth)| (f32::from(*source) - f32::from(smooth)).abs())
            .sum::<f32>()
            / samples.len() as f32;
        let noise_score = (1.0 - noise_residual / 48.0).clamp(0.0, 1.0);
        let glare_fraction = samples
            .iter()
            .zip(&gradients)
            .filter(|(sample, gradient)| {
                **sample >= self.config.overexposed_threshold
                    && **gradient <= self.config.glare_gradient_threshold
            })
            .count() as f32
            / samples.len() as f32;

        let mut report = QualityReport {
            vector: QualityVector(BTreeMap::new()),
            metrics: BTreeMap::new(),
            confidence: Confidence::new(0.7)?,
            warnings: Vec::new(),
            recommended_actions: Vec::new(),
            diagnostics: vec![
                "motion and occlusion are explicit scalar placeholders in MP3".into(),
                "glare is a conservative bright low-texture likelihood, not a material classifier"
                    .into(),
            ],
        };
        add_metric(
            &mut report,
            "blur_laplacian_variance",
            laplacian_variance,
            blur_score,
            0.75,
        )?;
        add_metric(
            &mut report,
            "blur_tenengrad_energy",
            gradient_energy,
            tenengrad_score,
            0.75,
        )?;
        add_metric(
            &mut report,
            "mean_luminance",
            mean_luminance,
            luminance_score(mean_luminance),
            0.85,
        )?;
        add_metric(
            &mut report,
            "underexposed_fraction",
            underexposed,
            1.0 - underexposed,
            0.9,
        )?;
        add_metric(
            &mut report,
            "overexposed_fraction",
            overexposed,
            1.0 - overexposed,
            0.9,
        )?;
        add_metric(
            &mut report,
            "dynamic_range",
            dynamic_range,
            (dynamic_range / 128.0).clamp(0.0, 1.0),
            0.8,
        )?;
        add_metric(
            &mut report,
            "contrast_stddev",
            luminance_stddev,
            (luminance_stddev / 64.0).clamp(0.0, 1.0),
            0.8,
        )?;
        add_metric(
            &mut report,
            "contrast_percentile_range",
            dynamic_range,
            contrast_score,
            0.8,
        )?;
        add_metric(
            &mut report,
            "saturation_clipped_fraction",
            clipped,
            1.0 - clipped,
            0.85,
        )?;
        add_metric(
            &mut report,
            "noise_residual",
            noise_residual,
            noise_score,
            0.55,
        )?;
        add_metric(
            &mut report,
            "glare_likelihood",
            glare_fraction,
            1.0 - glare_fraction,
            0.45,
        )?;
        add_metric(&mut report, "motion_placeholder", 0.0, 0.5, 0.0)?;
        add_metric(&mut report, "occlusion_placeholder", 0.0, 0.5, 0.0)?;

        if underexposed > 0.35 {
            warn(&mut report, "too_dark");
        }
        if overexposed > 0.35 {
            warn(&mut report, "too_bright");
        }
        if blur_score < 0.22 || tenengrad_score < 0.15 {
            warn(&mut report, "too_blurry");
            report.recommended_actions.push("hold_steady".into());
        }
        if contrast_score < 0.18 {
            warn(&mut report, "low_contrast");
        }
        if glare_fraction > 0.04 {
            warn(&mut report, "glare_detected");
        }
        Ok(report)
    }
}

fn packed_gray(image: &ImageBuffer) -> PerceptionResult<Vec<u8>> {
    let mut values = Vec::with_capacity(image.width() as usize * image.height() as usize);
    let view = image.view();
    for y in 0..image.height() {
        values.extend_from_slice(view.row(y)?);
    }
    Ok(values)
}

fn add_metric(
    report: &mut QualityReport,
    name: &str,
    raw_value: f32,
    score: f32,
    confidence: f32,
) -> PerceptionResult<()> {
    let normalized = score.clamp(0.0, 1.0);
    report.vector.0.insert(name.into(), normalized);
    report.metrics.insert(
        name.into(),
        QualityMeasurement {
            raw_value,
            normalized_score: Score::new(normalized)?,
            confidence: Confidence::new(confidence)?,
            diagnostics: Vec::new(),
        },
    );
    Ok(())
}

fn warn(report: &mut QualityReport, action: &str) {
    report.warnings.push(action.into());
    report.recommended_actions.push(action.into());
}

fn fraction(samples: &[u8], predicate: impl Fn(&u8) -> bool) -> f32 {
    samples.iter().filter(|sample| predicate(sample)).count() as f32 / samples.len() as f32
}

fn normalized_reference(value: f32, reference: f32) -> f32 {
    (value / (value + reference.max(1.0))).clamp(0.0, 1.0)
}

fn luminance_score(mean: f32) -> f32 {
    (1.0 - ((mean - 128.0).abs() / 128.0)).clamp(0.0, 1.0)
}
