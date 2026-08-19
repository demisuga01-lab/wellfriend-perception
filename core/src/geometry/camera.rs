//! Camera-model placeholders kept outside document-specific code.

use crate::{Point2, Transform3D};

/// Pinhole-camera intrinsic parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraIntrinsics {
    /// Horizontal focal length in pixel units.
    pub fx: f32,
    /// Vertical focal length in pixel units.
    pub fy: f32,
    /// Principal point.
    pub principal_point: Point2,
}

/// Lens distortion coefficients reserved for calibrated camera modules.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DistortionCoefficients {
    /// Radial coefficients in increasing order.
    pub radial: Vec<f32>,
    /// Tangential coefficients in increasing order.
    pub tangential: Vec<f32>,
}

/// Camera-to-world pose placeholder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPose {
    /** Camera-to-world transform. */
    pub transform: Transform3D,
}

/// Projection model selection for future calibration and reconstruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionModel {
    /** Perspective pinhole projection. */
    Pinhole,
    /** Orthographic projection. */
    Orthographic,
    /** Domain-defined projection. */
    Custom,
}
