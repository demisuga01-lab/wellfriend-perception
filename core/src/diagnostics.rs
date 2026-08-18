//! Lightweight diagnostics and timing records for pipeline observability.

use crate::PipelineStage;

/// Severity assigned to an engine diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    /// Trace-level implementation detail.
    Trace,
    /// Useful non-failure information.
    Info,
    /// Recoverable degradation or fallback.
    Warning,
    /// A stage could not complete its declared work.
    Error,
}

/// Stable, machine-readable diagnostic identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticCode(pub String);

/// Human-readable observation associated with a pipeline run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// Severity of this event.
    pub level: DiagnosticLevel,
    /// Stable code for filtering and product localization.
    pub code: DiagnosticCode,
    /// Detail intended for logs and development tools.
    pub message: String,
}

/// Timing captured for one pipeline stage in monotonic microseconds supplied by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageTiming {
    /// Stage represented by this timing record.
    pub stage: PipelineStage,
    /// Host-provided monotonic start time in microseconds.
    pub started_at_us: u64,
    /// Measured duration in microseconds.
    pub duration_us: u64,
}

/// Ordered diagnostics and timings for a single pipeline run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PipelineTrace {
    /// Events emitted by the pipeline or its selected domain pack.
    pub diagnostics: Vec<Diagnostic>,
    /// Stage timings emitted by the host runtime.
    pub timings: Vec<StageTiming>,
}

impl PipelineTrace {
    /// Appends a diagnostic without allocating an external logging dependency.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Appends a timing record.
    pub fn record_timing(&mut self, timing: StageTiming) {
        self.timings.push(timing);
    }
}
