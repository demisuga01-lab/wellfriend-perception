use crate::*;

/// Supplies observations from a camera, file system, network, or another input source.
pub trait InputProvider {
    /// Produces the next domain-neutral observation frame.
    fn next(&mut self, context: &PipelineContext) -> PerceptionResult<ObservationFrame>;
}
/// Measures generic and domain-defined quality signals.
pub trait QualityAnalyzer {
    /// Measures quality for an observation frame.
    fn analyze(
        &self,
        frame: &ObservationFrame,
        context: &PipelineContext,
    ) -> PerceptionResult<QualityReport>;
}
/// Produces independently attributable detection candidates.
pub trait Detector {
    /// Produces candidates without discarding source evidence.
    fn detect(
        &self,
        frame: &ObservationFrame,
        quality: &QualityReport,
        context: &PipelineContext,
    ) -> PerceptionResult<DetectionSet>;
}
/// Reconciles candidates from independent sources without discarding provenance.
pub trait FusionEngine {
    /// Fuses detector output.
    fn fuse(
        &self,
        detections: &[DetectionSet],
        context: &PipelineContext,
    ) -> PerceptionResult<FusionResult>;
}
/// Improves a selected candidate while preserving its uncertainty and source evidence.
pub trait Refiner {
    /// Refines fused evidence.
    fn refine(
        &self,
        fused: &FusionResult,
        context: &PipelineContext,
    ) -> PerceptionResult<RefinementResult>;
}
/// Maintains temporal evidence and capture readiness across observations.
pub trait TemporalEstimator {
    /// Updates state from a frame and refinement output.
    fn update(
        &mut self,
        frame: &ObservationFrame,
        result: &RefinementResult,
        context: &PipelineContext,
    ) -> PerceptionResult<TemporalState>;
}
/// Builds a declared geometry model from temporal and refined evidence.
pub trait Reconstructor {
    /// Reconstructs the selected model.
    fn reconstruct(
        &self,
        model: GeometryModel,
        refined: &RefinementResult,
        temporal: &TemporalState,
        context: &PipelineContext,
    ) -> PerceptionResult<ReconstructionResult>;
}
/// Converts observed defects or conditions into router inputs.
pub trait ConditionAnalyzer {
    /// Analyzes conditions from quality and reconstruction evidence.
    fn analyze_conditions(
        &self,
        quality: &QualityReport,
        reconstruction: &ReconstructionResult,
        context: &PipelineContext,
    ) -> PerceptionResult<ConditionVector>;
}
/// Executes one routed specialized operation.
pub trait Processor {
    /// Stable processor identifier.
    fn id(&self) -> &str;
    /// Declares routing capabilities and expected cost/benefit.
    fn capabilities(&self) -> ProcessorCapabilities;
    /// Processes an image under the selected condition plan.
    fn process(
        &self,
        image: &ImageBuffer,
        conditions: &ConditionVector,
        context: &PipelineContext,
    ) -> PerceptionResult<ProcessorResult>;
}
/// Assigns semantic regions and relationships to reconstructed output.
pub trait SemanticEngine {
    /// Interprets reconstructed imagery.
    fn interpret(
        &self,
        image: &ImageBuffer,
        reconstruction: &ReconstructionResult,
        context: &PipelineContext,
    ) -> PerceptionResult<SemanticResult>;
}
/// Serializes domain output into a stable schema.
pub trait Exporter {
    /// Exports semantic and reconstruction output.
    fn export(
        &self,
        semantic: &SemanticResult,
        reconstruction: &ReconstructionResult,
        context: &PipelineContext,
    ) -> PerceptionResult<StructuredOutput>;
}
/// Installs domain-specific behavior while retaining generic core contracts.
pub trait DomainPack {
    /// Stable pack identifier.
    fn id(&self) -> &str;
    /// Supported pipeline stages.
    fn supported_stages(&self) -> &[PipelineStage];
    /// Structured output schema identifier.
    fn output_schema(&self) -> &str;
}
