//! Lens-correction seam kept explicit before camera calibration is implemented.

use wellfriend_perception_core::{ImageBuffer, PerceptionResult};

/// Camera intrinsics referenced by a future calibrated correction stage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraIntrinsicsRef {
    /// Focal length in horizontal pixels.
    pub fx: f32,
    /// Focal length in vertical pixels.
    pub fy: f32,
    /// Principal point horizontal coordinate.
    pub cx: f32,
    /// Principal point vertical coordinate.
    pub cy: f32,
}

/// Radial distortion coefficients in the declaring camera model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadialDistortion {
    /// Quadratic radial coefficient.
    pub k1: f32,
    /// Quartic radial coefficient.
    pub k2: f32,
    /// Sextic radial coefficient.
    pub k3: f32,
}

/// Tangential distortion coefficients in the declaring camera model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TangentialDistortion {
    /// First tangential coefficient.
    pub p1: f32,
    /// Second tangential coefficient.
    pub p2: f32,
}

/// Lens model selected for a reconstruction request.
#[derive(Clone, Debug, PartialEq)]
pub enum LensCorrectionModel {
    /// No calibration data was supplied.
    None,
    /// A future radial correction with declared intrinsics.
    Radial {
        /// Camera parameters.
        intrinsics: CameraIntrinsicsRef,
        /// Radial coefficients.
        coefficients: RadialDistortion,
    },
    /// A future radial and tangential correction with declared intrinsics.
    RadialTangential {
        /// Camera parameters.
        intrinsics: CameraIntrinsicsRef,
        /// Radial coefficients.
        radial: RadialDistortion,
        /// Tangential coefficients.
        tangential: TangentialDistortion,
    },
}

/// Diagnostic emitted by a lens stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LensCorrectionStage {
    /// Whether image samples were changed.
    pub applied: bool,
    /// Stable diagnostic identifiers.
    pub diagnostics: Vec<String>,
}

/// No-op lens correction that records the boundary without pretending calibration.
#[derive(Clone, Debug, Default)]
pub struct NoOpLensCorrector;

impl NoOpLensCorrector {
    /// Carries the image forward and records whether correction remains deferred.
    pub fn apply(
        &self,
        input: &ImageBuffer,
        model: &LensCorrectionModel,
    ) -> PerceptionResult<(ImageBuffer, LensCorrectionStage)> {
        let diagnostic = match model {
            LensCorrectionModel::None => "lens_correction_not_requested",
            LensCorrectionModel::Radial { .. } | LensCorrectionModel::RadialTangential { .. } => {
                "lens_correction_model_supplied_but_scalar_correction_not_implemented"
            }
        };
        Ok((
            input.view().to_owned()?,
            LensCorrectionStage {
                applied: false,
                diagnostics: vec![diagnostic.into()],
            },
        ))
    }
}
