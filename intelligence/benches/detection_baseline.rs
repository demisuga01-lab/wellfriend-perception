//! Deterministic MP3 quality, detection, fusion, refinement, temporal, and readiness baselines.

use std::{hint::black_box, time::Instant};

use wellfriend_perception_core::{FrameIndex, benchmarks::BenchmarkRecord};
use wellfriend_perception_intelligence::{
    benchmarks::{SyntheticDocumentFixtureKind, synthetic_document_fixture},
    domains::document::ClassicalDocumentDetector,
    fusion::QuadFusionEngine,
    quality::ScalarQualityAnalyzer,
    readiness::{CaptureReadinessEngine, CaptureReadinessInput},
    refinement::QuadRefiner,
    temporal::QuadTemporalTracker,
};

const ITERATIONS: u64 = 30;

fn main() {
    let fixture =
        synthetic_document_fixture(SyntheticDocumentFixtureKind::Perspective).expect("fixture");
    let quality = ScalarQualityAnalyzer::default();
    let detector = ClassicalDocumentDetector::default();
    let fusion = QuadFusionEngine::default();
    let refiner = QuadRefiner::default();
    let report = quality.analyze(&fixture.image).expect("quality");
    let detection = detector.detect_image(&fixture.image).expect("detection");
    let fused = fusion
        .fuse(std::slice::from_ref(&detection.detections))
        .expect("fusion");
    let refined = refiner.refine(&fixture.image, &fused).expect("refinement");
    let quad = fixture.expected_quad.expect("document fixture");

    measure("quality_report_small_synthetic", || {
        let _ = black_box(quality.analyze(&fixture.image).expect("quality"));
    });
    measure("classical_document_detector", || {
        let _ = black_box(detector.detect_image(&fixture.image).expect("detector"));
    });
    measure("quad_fusion_candidates", || {
        let _ = black_box(
            fusion
                .fuse(std::slice::from_ref(&detection.detections))
                .expect("fusion"),
        );
    });
    measure("quad_refinement", || {
        let _ = black_box(
            refiner
                .refine_quad(&fixture.image, quad, fused.confidence)
                .expect("refinement"),
        );
    });
    measure("temporal_update_sequence", || {
        let mut tracker = QuadTemporalTracker::default();
        for index in 0..8 {
            let _ = tracker
                .update(FrameIndex(index), Some(quad))
                .expect("temporal");
        }
        let _ = black_box(tracker);
    });
    measure("capture_readiness", || {
        let mut tracker = QuadTemporalTracker::default();
        let state = (0..6)
            .map(|index| {
                tracker
                    .update(FrameIndex(index), Some(quad))
                    .expect("temporal")
            })
            .last()
            .expect("state")
            .state;
        let _ = black_box(
            CaptureReadinessEngine::default().evaluate(CaptureReadinessInput {
                quality: &report,
                fusion: &fused,
                refinement: &refined,
                temporal: &state,
                image_shape: fixture.image.shape(),
            }),
        );
    });
}

fn measure(operation: &str, mut action: impl FnMut()) {
    let started = Instant::now();
    for _ in 0..ITERATIONS {
        action();
    }
    let record = BenchmarkRecord::synthetic_baseline(
        "document",
        "mp3-generated-perspective-160x120",
        operation,
        ITERATIONS,
        started.elapsed().as_nanos(),
    );
    println!("{}", record.to_json_line());
}
