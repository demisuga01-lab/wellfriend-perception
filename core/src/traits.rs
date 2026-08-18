use crate::*;

/// Supplies observations from a camera, file system, network, or another input source.
pub trait InputProvider {
    fn next(&mut self, context: &PipelineContext) -> Result<ObservationFrame, String>;
}
/// Measures generic and domain-defined quality signals.
pub trait QualityAnalyzer {
    fn analyze(
        &self,
        frame: &ObservationFrame,
        context: &PipelineContext,
    ) -> Result<QualityReport, String>;
}
/// Produces independently attributable detection candidates.
pub trait Detector {
    fn detect(
        &self,
        frame: &ObservationFrame,
        quality: &QualityReport,
        context: &PipelineContext,
    ) -> Result<DetectionSet, String>;
}
/// Reconciles candidates from independent sources without discarding provenance.
pub trait FusionEngine {
    fn fuse(
        &self,
        detections: &[DetectionSet],
        context: &PipelineContext,
    ) -> Result<FusionResult, String>;
}
/// Improves a selected candidate while preserving its uncertainty and source evidence.
pub trait Refiner {
    fn refine(
        &self,
        fused: &FusionResult,
        context: &PipelineContext,
    ) -> Result<RefinementResult, String>;
}
/// Maintains temporal evidence and capture readiness across observations.
pub trait TemporalEstimator {
    fn update(
        &mut self,
        frame: &ObservationFrame,
        result: &RefinementResult,
        context: &PipelineContext,
    ) -> Result<TemporalState, String>;
}
/// Builds a declared geometry model from temporal and refined evidence.
pub trait Reconstructor {
    fn reconstruct(
        &self,
        model: GeometryModel,
        refined: &RefinementResult,
        temporal: &TemporalState,
        context: &PipelineContext,
    ) -> Result<ReconstructionResult, String>;
}
/// Converts observed defects or conditions into router inputs.
pub trait ConditionAnalyzer {
    fn analyze_conditions(
        &self,
        quality: &QualityReport,
        reconstruction: &ReconstructionResult,
        context: &PipelineContext,
    ) -> Result<ConditionVector, String>;
}
/// Executes one routed specialized operation.
pub trait Processor {
    fn id(&self) -> &str;
    fn capabilities(&self) -> ProcessorCapabilities;
    fn process(
        &self,
        image: &ImageBuffer,
        conditions: &ConditionVector,
        context: &PipelineContext,
    ) -> Result<ProcessorResult, String>;
}
/// Assigns semantic regions and relationships to reconstructed output.
pub trait SemanticEngine {
    fn interpret(
        &self,
        image: &ImageBuffer,
        reconstruction: &ReconstructionResult,
        context: &PipelineContext,
    ) -> Result<SemanticResult, String>;
}
/// Serializes domain output into a stable schema.
pub trait Exporter {
    fn export(
        &self,
        semantic: &SemanticResult,
        reconstruction: &ReconstructionResult,
        context: &PipelineContext,
    ) -> Result<StructuredOutput, String>;
}
/// Installs domain-specific behavior while retaining generic core contracts.
pub trait DomainPack {
    fn id(&self) -> &str;
    fn supported_stages(&self) -> &[PipelineStage];
    fn output_schema(&self) -> &str;
}
