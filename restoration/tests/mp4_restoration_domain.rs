use wellfriend_perception_core::{
    Confidence, ImageBuffer, PixelFormat, QualityMeasurement, QualityReport, Score,
};
use wellfriend_perception_restoration::{
    ConditionAnalyzerInput, ConditionEvidence, ConditionKind, ConditionVector, DeviceClass,
    DocumentDomainPack, DocumentFilterGraph, DocumentFilterPreset, DomainPackRegistry, ProcessorId,
    RestorationInput, RestorationProcessor, ScalarConditionAnalyzer, ScalarRestorationProcessor,
    SpecialistRouter, industrial_domain_pack, medical_research_domain_pack, satellite_domain_pack,
};

fn image() -> ImageBuffer {
    ImageBuffer::new(
        8,
        6,
        PixelFormat::Gray8,
        [100, 105, 110, 115, 120, 125, 130, 135].repeat(6),
    )
    .unwrap()
}

fn conditions() -> ConditionVector {
    let mut vector = ConditionVector::default();
    vector.insert(
        ConditionKind::LowContrast,
        ConditionEvidence {
            score: Score::new(0.9).unwrap(),
            confidence: Confidence::new(0.8).unwrap(),
            evidence: Vec::new(),
            sources: vec!["test".into()],
            recommended_processors: vec![ProcessorId::new("background_normalization")],
            diagnostics: Vec::new(),
        },
    );
    vector.insert(
        ConditionKind::Noise,
        ConditionEvidence {
            score: Score::new(0.7).unwrap(),
            confidence: Confidence::new(0.8).unwrap(),
            evidence: Vec::new(),
            sources: vec!["test".into()],
            recommended_processors: vec![ProcessorId::new("denoise")],
            diagnostics: Vec::new(),
        },
    );
    vector
}

#[test]
fn condition_analyzer_preserves_scalar_and_placeholder_boundaries() {
    let mut report = QualityReport::default();
    report.metrics.insert(
        "blur_laplacian_variance".into(),
        QualityMeasurement {
            raw_value: 0.1,
            normalized_score: Score::new(0.1).unwrap(),
            confidence: Confidence::new(0.8).unwrap(),
            diagnostics: Vec::new(),
        },
    );
    report.metrics.insert(
        "glare_likelihood".into(),
        QualityMeasurement {
            raw_value: 0.7,
            normalized_score: Score::new(0.3).unwrap(),
            confidence: Confidence::new(0.5).unwrap(),
            diagnostics: Vec::new(),
        },
    );
    let output = ScalarConditionAnalyzer.analyze(&ConditionAnalyzerInput {
        quality: &report,
        fusion: None,
        refinement: None,
        temporal: None,
        reconstruction: None,
        domain_metadata: Default::default(),
    });
    assert!(output.score(ConditionKind::Blur) > 0.8);
    assert!(output.score(ConditionKind::Glare) > 0.6);
    assert!(output.entries[&ConditionKind::Shadow].diagnostics[0].contains("uncalibrated"));
}

#[test]
fn router_explains_low_device_skips_and_avoids_conflicts() {
    let router = SpecialistRouter::default();
    let low = router
        .plan(&conditions(), DocumentFilterPreset::Clean, DeviceClass::Low)
        .unwrap();
    assert!(
        low.plan
            .steps
            .iter()
            .all(|step| step.cost.latency_units <= 1)
    );
    assert!(!low.plan.skipped.is_empty());
    let bw = router
        .plan(
            &conditions(),
            DocumentFilterPreset::BlackAndWhite,
            DeviceClass::Mid,
        )
        .unwrap();
    assert!(
        !bw.plan
            .steps
            .iter()
            .any(|step| step.processor.as_str() == "unsharp")
    );
}

#[test]
fn scalar_processors_and_filters_have_real_baseline_behavior() {
    let source = image();
    let grayscale = ScalarRestorationProcessor::Grayscale
        .process(
            &RestorationInput {
                image: source.clone(),
            },
            &Default::default(),
        )
        .unwrap();
    assert_eq!(grayscale.image.pixel_format(), PixelFormat::Gray8);
    let normalized = ScalarRestorationProcessor::BrightnessContrast
        .process(
            &RestorationInput {
                image: source.clone(),
            },
            &Default::default(),
        )
        .unwrap();
    assert!(normalized.image.as_bytes().iter().min() < source.as_bytes().iter().min());
    let gamma = ScalarRestorationProcessor::Gamma { gamma: 0.5 }
        .process(
            &RestorationInput {
                image: source.clone(),
            },
            &Default::default(),
        )
        .unwrap();
    assert_ne!(gamma.image, source);
    let sharpened = ScalarRestorationProcessor::Unsharp { amount: 0.8 }
        .process(
            &RestorationInput {
                image: source.clone(),
            },
            &Default::default(),
        )
        .unwrap();
    assert_eq!(sharpened.image.shape(), source.shape());
    let binary = ScalarRestorationProcessor::Binarize {
        mode: wellfriend_perception_restoration::BinarizationMode::Otsu,
    }
    .process(
        &RestorationInput {
            image: source.clone(),
        },
        &Default::default(),
    )
    .unwrap();
    assert!(
        binary
            .image
            .as_bytes()
            .iter()
            .all(|value| *value == 0 || *value == 255)
    );
    let original = DocumentFilterGraph::default()
        .apply(
            DocumentFilterPreset::Original,
            &source,
            &conditions(),
            DeviceClass::Mid,
        )
        .unwrap();
    assert_eq!(original.image, source);
    let clean = DocumentFilterGraph::default()
        .apply(
            DocumentFilterPreset::Clean,
            &source,
            &conditions(),
            DeviceClass::Mid,
        )
        .unwrap();
    assert!(!clean.applied_processors.is_empty());
}

#[test]
fn domain_packs_register_without_changing_core_semantics() {
    let mut registry = DomainPackRegistry::default();
    registry
        .register(Box::new(DocumentDomainPack::new()))
        .unwrap();
    registry
        .register(Box::new(medical_research_domain_pack()))
        .unwrap();
    registry
        .register(Box::new(satellite_domain_pack()))
        .unwrap();
    registry
        .register(Box::new(industrial_domain_pack()))
        .unwrap();
    assert_eq!(registry.get("document").unwrap().id(), "document");
    assert_eq!(
        registry.get("medical_research").unwrap().id(),
        "medical_research"
    );
    assert!(registry.get("unknown").is_err());
}
