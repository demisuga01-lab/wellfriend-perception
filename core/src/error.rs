//! Error types shared by the domain-neutral perception engine.

use core::fmt;

/// Result alias used by fallible Wellfriend perception APIs.
pub type PerceptionResult<T> = Result<T, PerceptionError>;

/// Failures that callers can handle without relying on panics or string matching.
#[derive(Clone, Debug, PartialEq)]
pub enum PerceptionError {
    /// Width, height, or another shape component is invalid.
    InvalidDimensions { width: u32, height: u32 },
    /// Supplied bytes cannot represent the declared image buffer.
    InvalidBuffer { expected: usize, actual: usize },
    /// A row stride is smaller than the minimum required width.
    StrideMismatch { minimum: usize, actual: usize },
    /// A requested format is valid but unsupported by an operation.
    UnsupportedFormat {
        operation: &'static str,
        format: String,
    },
    /// A coordinate, region, or index is outside of valid bounds.
    OutOfBounds { reason: String },
    /// Checked arithmetic overflowed while calculating a buffer or index.
    Overflow,
    /// A finite numerical result could not be produced.
    NumericFailure { reason: String },
    /// Input geometry has insufficient rank, area, or separation.
    DegenerateGeometry { reason: String },
    /// A matrix has no numerically stable inverse.
    NonInvertibleMatrix,
    /// An algorithm did not receive enough input samples.
    InsufficientPoints { required: usize, actual: usize },
    /// A confidence-like score was non-finite or outside the closed unit interval.
    InvalidConfidence { value: f32 },
    /// An API is deliberately reserved for a future implementation.
    UnsupportedOperation { operation: &'static str },
}

impl fmt::Display for PerceptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions { width, height } => {
                write!(formatter, "invalid dimensions {width}x{height}")
            }
            Self::InvalidBuffer { expected, actual } => write!(
                formatter,
                "invalid buffer length: expected {expected}, got {actual}"
            ),
            Self::StrideMismatch { minimum, actual } => {
                write!(formatter, "invalid stride: minimum {minimum}, got {actual}")
            }
            Self::UnsupportedFormat { operation, format } => {
                write!(formatter, "{operation} does not support {format}")
            }
            Self::OutOfBounds { reason } => write!(formatter, "out of bounds: {reason}"),
            Self::Overflow => formatter.write_str("checked arithmetic overflow"),
            Self::NumericFailure { reason } => write!(formatter, "numeric failure: {reason}"),
            Self::DegenerateGeometry { reason } => {
                write!(formatter, "degenerate geometry: {reason}")
            }
            Self::NonInvertibleMatrix => formatter.write_str("matrix is non-invertible"),
            Self::InsufficientPoints { required, actual } => write!(
                formatter,
                "insufficient points: need {required}, got {actual}"
            ),
            Self::InvalidConfidence { value } => write!(formatter, "invalid confidence {value}"),
            Self::UnsupportedOperation { operation } => {
                write!(formatter, "unsupported operation: {operation}")
            }
        }
    }
}

impl std::error::Error for PerceptionError {}
