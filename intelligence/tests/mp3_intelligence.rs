use std::collections::BTreeMap;

use wellfriend_perception_core::{
    Confidence, DetectionGeometry, DetectionSet, DetectionSource, FusionResult, ImageBuffer,
    ImageShape, PixelFormat, QualityMeasurement, QualityReport, QualityVector, RefinementResult,
    Score,
};
use wellfriend_perception_image::box_blur_gray;
use wellfriend_perception_intelligence::{
    benchmarks::{SyntheticDocumentFixtureKind, synthetic_document_fixture},
    detection::{document_quad_candidate, manual_quad_candidate},
    domains::document::{
        ClassicalDocumentDetector, analyze_document_quality, apply_document_quality_extensions,
    },
    fusion::{QuadFusionEngine, quad_agreement, quad_iou},
    quality::ScalarQualityAnalyzer,
    readiness::{CaptureGuidance, CaptureReadiness, CaptureReadinessEngine, CaptureReadinessInput},
    refinement::QuadRefiner,
    temporal::QuadTemporalTracker,
};

fn score(value: f32) -> Score {
    Score::new(value).unwrap()
}

fn candidate(
    quad: wellfriend_perception_core::Quad,
    source: DetectionSource,
    score_value: f32,
) -> wellfriend_perception_core::DetectionCandidate {
    document_quad_candidate(source, quad, score_value, "test-detector").unwrap()
}

fn set(candidates: Vec<wellfriend_perception_core::DetectionCandidate>) -> DetectionSet {
    DetectionSet {
        candidates,
        detector_id: Some("test".into()),
        diagnostics: Vec::new(),
    }
}

fn quad() -> wellfriend_perception_core::Quad {
    synthetic_document_fixture(SyntheticDocumentFixtureKind::PlainCentered)
        .unwrap()
        .expected_quad
        .unwrap()
}

#[test]
fn quality_metrics_distinguish_blur_exposure_contrast_saturation_and_glare() {
    let fixture = synthetic_document_fixture(SyntheticDocumentFixtureKind::PlainCentered).unwrap();
    let analyzer = ScalarQualityAnalyzer::default();
    let sharp = analyzer.analyze(&fixture.image).unwrap();
    let blurred = box_blur_gray(&box_blur_gray(&fixture.image).unwrap()).unwrap();
    let soft = analyzer.analyze(&blurred).unwrap();
    assert!(
        sharp.metrics["blur_laplacian_variance"].raw_value
            > soft.metrics["blur_laplacian_variance"].raw_value
    );
    let black = ImageBuffer::new(32, 32, PixelFormat::Gray8, vec![0; 1024]).unwrap();
    let white = ImageBuffer::new(32, 32, PixelFormat::Gray8, vec![255; 1024]).unwrap();
    assert!(
        analyzer
            .analyze(&black)
            .unwrap()
            .warnings
            .iter()
            .any(|item| item == "too_dark")
    );
    assert!(
        analyzer
            .analyze(&white)
            .unwrap()
            .warnings
            .iter()
            .any(|item| item == "too_bright")
    );
    let flat = ImageBuffer::new(32, 32, PixelFormat::Gray8, vec![128; 1024]).unwrap();
    assert!(
        analyzer.analyze(&fixture.image).unwrap().metrics["contrast_percentile_range"]
            .normalized_score
            .value()
            > analyzer.analyze(&flat).unwrap().metrics["contrast_percentile_range"]
                .normalized_score
                .value()
    );
    assert!(
        analyzer.analyze(&white).unwrap().metrics["saturation_clipped_fraction"].raw_value > 0.0
    );
    let glare = analyzer.analyze(&fixture.image).unwrap();
    assert!(glare.metrics["glare_likelihood"].raw_value > 0.0);
}

#[test]
fn document_quality_extensions_emit_frame_guidance() {
    let fixture = synthetic_document_fixture(SyntheticDocumentFixtureKind::PlainCentered).unwrap();
    let (report, extensions) =
        analyze_document_quality(&fixture.image, fixture.expected_quad).unwrap();
    assert!(extensions.page_coverage > 0.4);
    assert_eq!(extensions.page_visibility, 1.0);
    assert!(
        report
            .metrics
            .contains_key("document_shadow_likelihood_baseline")
    );
    let partial = synthetic_document_fixture(SyntheticDocumentFixtureKind::PartialCutOff).unwrap();
    let mut report = ScalarQualityAnalyzer::default()
        .analyze(&partial.image)
        .unwrap();
    apply_document_quality_extensions(&mut report, &partial.image, partial.expected_quad).unwrap();
    assert!(
        report
            .warnings
            .iter()
            .any(|item| item == "document_cut_off")
    );
}

#[test]
fn classical_detector_finds_synthetic_pages_and_rejects_no_document() {
    let detector = ClassicalDocumentDetector::default();
    for kind in [
        SyntheticDocumentFixtureKind::PlainCentered,
        SyntheticDocumentFixtureKind::Rotated,
        SyntheticDocumentFixtureKind::Perspective,
        SyntheticDocumentFixtureKind::LowContrast,
    ] {
        let fixture = synthetic_document_fixture(kind).unwrap();
        let output = detector.detect_image(&fixture.image).unwrap();
        assert!(
            !output.detections.candidates.is_empty(),
            "fixture {kind:?} was not detected"
        );
    }
    let empty = synthetic_document_fixture(SyntheticDocumentFixtureKind::NoDocument).unwrap();
    assert!(
        detector
            .detect_image(&empty.image)
            .unwrap()
            .detections
            .candidates
            .is_empty()
    );
}

#[test]
fn candidate_validation_penalizes_bad_geometry_and_records_heuristic_score() {
    let tiny = wellfriend_perception_core::Quad {
        points: [
            wellfriend_perception_core::Point2::new(1.0, 1.0),
            wellfriend_perception_core::Point2::new(3.0, 1.0),
            wellfriend_perception_core::Point2::new(3.0, 3.0),
            wellfriend_perception_core::Point2::new(1.0, 3.0),
        ],
    };
    let large = quad();
    let detector = ClassicalDocumentDetector::default();
    let fixture = synthetic_document_fixture(SyntheticDocumentFixtureKind::PlainCentered).unwrap();
    let detected = detector.detect_image(&fixture.image).unwrap();
    assert!(detected.detections.candidates[0].score.value() > 0.15);
    assert!(
        detected.detections.candidates[0].attributes["edge_support"]
            .parse::<f32>()
            .unwrap()
            > 0.2
    );
    let distractors =
        synthetic_document_fixture(SyntheticDocumentFixtureKind::MultipleDistractors).unwrap();
    let multi = detector.detect_image(&distractors.image).unwrap();
    assert_eq!(
        multi.detections.candidates.len(),
        1,
        "tiny rectangle distractor must be rejected"
    );
    assert!(document_quad_candidate(DetectionSource::Classical, tiny, 0.1, "tiny").is_ok());
    let concave = wellfriend_perception_core::Quad {
        points: [
            wellfriend_perception_core::Point2::new(0.0, 0.0),
            wellfriend_perception_core::Point2::new(10.0, 0.0),
            wellfriend_perception_core::Point2::new(2.0, 2.0),
            wellfriend_perception_core::Point2::new(0.0, 10.0),
        ],
    };
    assert!(document_quad_candidate(DetectionSource::Classical, concave, 0.9, "bad").is_err());
    assert!(large.validate().is_ok());
}

#[test]
fn fusion_combines_agreement_rejects_outliers_and_honors_manual_override() {
    let base = quad();
    let nearby = wellfriend_perception_core::Quad {
        points: base
            .points
            .map(|point| wellfriend_perception_core::Point2::new(point.x + 1.0, point.y - 1.0)),
    };
    let outlier = wellfriend_perception_core::Quad {
        points: base
            .points
            .map(|point| wellfriend_perception_core::Point2::new(point.x + 60.0, point.y)),
    };
    let engine = QuadFusionEngine::default();
    let fused = engine
        .fuse(&[
            set(vec![candidate(base, DetectionSource::Classical, 0.8)]),
            set(vec![candidate(nearby, DetectionSource::Ml, 0.9)]),
            set(vec![candidate(outlier, DetectionSource::Temporal, 0.8)]),
        ])
        .unwrap();
    assert_eq!(fused.contributing_sources.len(), 2);
    assert!(fused.rejected_sources.contains(&DetectionSource::Temporal));
    assert!(fused.disagreement_score.value() < 0.2);
    assert!(quad_iou(base, nearby) > 0.9);
    assert!(quad_agreement(base, outlier) < quad_agreement(base, nearby));

    let manual = wellfriend_perception_core::Quad {
        points: base
            .points
            .map(|point| wellfriend_perception_core::Point2::new(point.x + 4.0, point.y)),
    };
    let override_result = engine
        .fuse(&[
            set(vec![candidate(base, DetectionSource::Classical, 0.9)]),
            set(vec![manual_quad_candidate(manual).unwrap()]),
        ])
        .unwrap();
    assert_eq!(
        override_result.contributing_sources,
        vec![DetectionSource::Manual]
    );
    assert!(matches!(
        override_result.fused_geometry,
        Some(DetectionGeometry::Quad(_))
    ));
}

#[test]
fn refinement_improves_or_safely_bounds_noisy_coarse_quad() {
    let fixture = synthetic_document_fixture(SyntheticDocumentFixtureKind::PlainCentered).unwrap();
    let expected = fixture.expected_quad.unwrap();
    let coarse = wellfriend_perception_core::Quad {
        points: expected
            .points
            .map(|point| wellfriend_perception_core::Point2::new(point.x + 3.0, point.y - 2.0)),
    };
    let refiner = QuadRefiner::default();
    let result = refiner
        .refine_quad(&fixture.image, coarse, Confidence::new(0.8).unwrap())
        .unwrap();
    let refined = match result.refined_geometry.unwrap() {
        DetectionGeometry::Quad(quad) => quad,
        _ => unreachable!(),
    };
    let coarse_error = mean_error(coarse, expected);
    let refined_error = mean_error(refined, expected);
    assert!(refined_error <= coarse_error || result.refinement_delta == 0.0);
    let flat = ImageBuffer::new(32, 32, PixelFormat::Gray8, vec![100; 1024]).unwrap();
    let safe = refiner
        .refine_quad(&flat, coarse, Confidence::new(0.8).unwrap())
        .unwrap();
    assert_eq!(safe.refinement_delta, 0.0);
}

#[test]
fn temporal_tracker_separates_stable_jitter_lost_and_smoothed_sequences() {
    let base = quad();
    let mut stable = QuadTemporalTracker::default();
    for index in 0..6 {
        stable
            .update(wellfriend_perception_core::FrameIndex(index), Some(base))
            .unwrap();
    }
    let stable_state = stable
        .update(wellfriend_perception_core::FrameIndex(7), Some(base))
        .unwrap()
        .state;
    assert!(stable_state.stable);
    let mut jittery = QuadTemporalTracker::default();
    for index in 0..6 {
        let offset = if index % 2 == 0 { 14.0 } else { -14.0 };
        let noisy = wellfriend_perception_core::Quad {
            points: base
                .points
                .map(|point| wellfriend_perception_core::Point2::new(point.x + offset, point.y)),
        };
        jittery
            .update(wellfriend_perception_core::FrameIndex(index), Some(noisy))
            .unwrap();
    }
    let jittery_state = jittery
        .update(wellfriend_perception_core::FrameIndex(7), Some(base))
        .unwrap()
        .state;
    assert!(jittery_state.stability_score.value() < stable_state.stability_score.value());
    for index in 8..14 {
        jittery
            .update(wellfriend_perception_core::FrameIndex(index), None)
            .unwrap();
    }
    assert!(jittery.temporal_candidate().unwrap().is_none());
}

#[test]
fn capture_readiness_handles_missing_blurry_unstable_good_and_cutoff_inputs() {
    let engine = CaptureReadinessEngine::default();
    let shape = ImageShape::new(160, 120).unwrap();
    let empty_fusion = FusionResult::default();
    let empty_refinement = RefinementResult::default();
    let temporal = wellfriend_perception_core::TemporalState::default();
    let missing = engine.evaluate(CaptureReadinessInput {
        quality: &QualityReport::default(),
        fusion: &empty_fusion,
        refinement: &empty_refinement,
        temporal: &temporal,
        image_shape: shape,
    });
    assert_eq!(missing.readiness, CaptureReadiness::NotReady);
    assert_eq!(missing.guidance, vec![CaptureGuidance::NoDocument]);

    let good_quality = quality_report(&[]);
    let stable = wellfriend_perception_core::TemporalState {
        stable: true,
        confidence: Confidence::new(0.95).unwrap(),
        track_id: Some(1),
        frame_index: None,
        velocity: None,
        stability_score: score(0.96),
        lost_frames: 0,
        diagnostics: Vec::new(),
    };
    let fused = FusionResult {
        fused_geometry: Some(DetectionGeometry::Quad(quad())),
        confidence: Confidence::new(0.95).unwrap(),
        disagreement_score: score(0.02),
        ..Default::default()
    };
    let refined = RefinementResult {
        refined_geometry: Some(DetectionGeometry::Quad(quad())),
        confidence: Confidence::new(0.95).unwrap(),
        ..Default::default()
    };
    let ready = engine.evaluate(CaptureReadinessInput {
        quality: &good_quality,
        fusion: &fused,
        refinement: &refined,
        temporal: &stable,
        image_shape: shape,
    });
    assert!(matches!(
        ready.readiness,
        CaptureReadiness::Ready | CaptureReadiness::CaptureNow
    ));
    let blurry_quality = quality_report(&["too_blurry"]);
    let blurry = engine.evaluate(CaptureReadinessInput {
        quality: &blurry_quality,
        fusion: &fused,
        refinement: &refined,
        temporal: &stable,
        image_shape: shape,
    });
    assert_eq!(blurry.readiness, CaptureReadiness::NotReady);
    let cutoff_quality = quality_report(&["document_cut_off"]);
    let cutoff = engine.evaluate(CaptureReadinessInput {
        quality: &cutoff_quality,
        fusion: &fused,
        refinement: &refined,
        temporal: &stable,
        image_shape: shape,
    });
    assert!(cutoff.guidance.contains(&CaptureGuidance::DocumentCutOff));
}

fn mean_error(
    left: wellfriend_perception_core::Quad,
    right: wellfriend_perception_core::Quad,
) -> f32 {
    left.points
        .iter()
        .zip(right.points)
        .map(|(a, b)| a.distance(b))
        .sum::<f32>()
        / 4.0
}

fn quality_report(warnings: &[&str]) -> QualityReport {
    let mut metrics = BTreeMap::new();
    for name in [
        "mean_luminance",
        "blur_laplacian_variance",
        "blur_tenengrad_energy",
        "contrast_percentile_range",
    ] {
        metrics.insert(
            name.into(),
            QualityMeasurement {
                raw_value: 1.0,
                normalized_score: score(0.95),
                confidence: Confidence::new(0.9).unwrap(),
                diagnostics: Vec::new(),
            },
        );
    }
    QualityReport {
        vector: QualityVector(BTreeMap::new()),
        metrics,
        confidence: Confidence::new(0.9).unwrap(),
        warnings: warnings.iter().map(|value| (*value).into()).collect(),
        recommended_actions: Vec::new(),
        diagnostics: Vec::new(),
    }
}
