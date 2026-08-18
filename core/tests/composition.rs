use wellfriend_perception_core::*;

struct Input;
impl InputProvider for Input {
    fn next(&mut self, _: &PipelineContext) -> Result<ObservationFrame, String> {
        Ok(ObservationFrame::default())
    }
}
struct Quality;
impl QualityAnalyzer for Quality {
    fn analyze(&self, _: &ObservationFrame, _: &PipelineContext) -> Result<QualityReport, String> {
        Ok(QualityReport::default())
    }
}
struct Detect;
impl Detector for Detect {
    fn detect(
        &self,
        _: &ObservationFrame,
        _: &QualityReport,
        _: &PipelineContext,
    ) -> Result<DetectionSet, String> {
        Ok(DetectionSet::default())
    }
}
struct Fuse;
impl FusionEngine for Fuse {
    fn fuse(&self, sources: &[DetectionSet], _: &PipelineContext) -> Result<FusionResult, String> {
        Ok(FusionResult {
            candidates: sources.first().cloned().unwrap_or_default(),
            diagnostics: vec!["aggregated".into()],
        })
    }
}
struct Refine;
impl Refiner for Refine {
    fn refine(
        &self,
        fused: &FusionResult,
        _: &PipelineContext,
    ) -> Result<RefinementResult, String> {
        Ok(RefinementResult {
            candidates: fused.candidates.clone(),
            diagnostics: vec![],
        })
    }
}
struct Temporal;
impl TemporalEstimator for Temporal {
    fn update(
        &mut self,
        _: &ObservationFrame,
        _: &RefinementResult,
        _: &PipelineContext,
    ) -> Result<TemporalState, String> {
        Ok(TemporalState {
            stable: true,
            confidence: 1.0,
            diagnostics: vec![],
        })
    }
}

#[test]
fn generic_stage_contracts_compose() {
    let mut input = Input;
    let quality = Quality;
    let detector = Detect;
    let fusion = Fuse;
    let refiner = Refine;
    let mut temporal = Temporal;
    let mut pipeline = Pipeline {
        input: &mut input,
        quality: &quality,
        detector: &detector,
        fusion: &fusion,
        refiner: &refiner,
        temporal: &mut temporal,
    };
    let (_, refined, state) = pipeline
        .observe(&PipelineContext {
            run_id: "test".into(),
            domain: "document".into(),
            attributes: Default::default(),
        })
        .unwrap();
    assert!(refined.candidates.candidates.is_empty());
    assert!(state.stable);
}

#[test]
fn image_buffer_rejects_invalid_storage() {
    assert!(ImageBuffer::new(2, 2, PixelFormat::Rgb8, vec![0; 11]).is_err());
}
