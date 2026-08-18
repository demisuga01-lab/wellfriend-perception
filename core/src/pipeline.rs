use crate::*;

/// Minimal pipeline composition harness used by apps and integration tests.
pub struct Pipeline<'a> {
    pub input: &'a mut dyn InputProvider,
    pub quality: &'a dyn QualityAnalyzer,
    pub detector: &'a dyn Detector,
    pub fusion: &'a dyn FusionEngine,
    pub refiner: &'a dyn Refiner,
    pub temporal: &'a mut dyn TemporalEstimator,
}

impl Pipeline<'_> {
    /// Runs the generic observation-to-temporal portion of the architecture.
    pub fn observe(
        &mut self,
        context: &PipelineContext,
    ) -> Result<(QualityReport, RefinementResult, TemporalState), String> {
        let frame = self.input.next(context)?;
        let quality = self.quality.analyze(&frame, context)?;
        let detections = self.detector.detect(&frame, &quality, context)?;
        let fused = self.fusion.fuse(&[detections], context)?;
        let refined = self.refiner.refine(&fused, context)?;
        let temporal = self.temporal.update(&frame, &refined, context)?;
        Ok((quality, refined, temporal))
    }
}
