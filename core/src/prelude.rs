//! Curated stable imports for consumers of the generic engine foundation.

pub use crate::{
    Confidence, Diagnostic, ImageBuffer, ImageShape, ImageView, PerceptionError, PerceptionResult,
    PixelFormat, Point2, RegionOfInterest, Transform2D,
    geometry::{estimate_homography_4pt, least_squares_line_fit, ransac_line_fit},
    math::{Mat2, Mat3, Mat4, Vec2, Vec3},
};
