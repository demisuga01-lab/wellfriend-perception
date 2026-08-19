//! Surface and dense-warp contracts for curved-page and future 2.5D reconstruction.

use wellfriend_perception_core::{
    Confidence, DenseWarpField, ImageBuffer, PerceptionError, PerceptionResult, Point2, Point3,
    geometry::{SamplingMode, WarpBorder, remap},
};

/// A sampled point that relates output-surface position to source-image position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceControlPoint {
    /// Normalized parameter-space coordinate in [0, 1] for a regular grid.
    pub parameter: Point2,
    /// Source-image coordinate sampled at this grid point.
    pub source: Point2,
    /// Optional model-space position; z remains an explicit surface seam.
    pub surface: Point3,
}

/// A regular surface-control grid.
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceGrid {
    /// Number of control columns, at least two.
    pub columns: u32,
    /// Number of control rows, at least two.
    pub rows: u32,
    /// Row-major control points.
    pub control_points: Vec<SurfaceControlPoint>,
}

impl SurfaceGrid {
    /// Validates grid dimensions, count, finiteness, and normalized parameters.
    pub fn validate(&self) -> PerceptionResult<()> {
        if self.columns < 2 || self.rows < 2 {
            return Err(PerceptionError::InvalidDimensions {
                width: self.columns,
                height: self.rows,
            });
        }
        let expected = self
            .columns
            .checked_mul(self.rows)
            .ok_or(PerceptionError::Overflow)? as usize;
        if self.control_points.len() != expected {
            return Err(PerceptionError::InvalidBuffer {
                expected,
                actual: self.control_points.len(),
            });
        }
        if self.control_points.iter().any(|point| {
            ![
                point.parameter.x,
                point.parameter.y,
                point.source.x,
                point.source.y,
                point.surface.x,
                point.surface.y,
                point.surface.z,
            ]
            .iter()
            .all(|value| value.is_finite())
                || !(0.0..=1.0).contains(&point.parameter.x)
                || !(0.0..=1.0).contains(&point.parameter.y)
        }) {
            return Err(PerceptionError::NumericFailure {
                reason: "surface controls must be finite with normalized parameters".into(),
            });
        }
        Ok(())
    }

    /// Creates an identity source-coordinate grid for a source image.
    pub fn identity(
        columns: u32,
        rows: u32,
        source_width: u32,
        source_height: u32,
    ) -> PerceptionResult<Self> {
        if source_width == 0 || source_height == 0 {
            return Err(PerceptionError::InvalidDimensions {
                width: source_width,
                height: source_height,
            });
        }
        if columns < 2 || rows < 2 {
            return Err(PerceptionError::InvalidDimensions {
                width: columns,
                height: rows,
            });
        }
        let mut control_points = Vec::with_capacity((columns * rows) as usize);
        for y in 0..rows {
            for x in 0..columns {
                let u = x as f32 / (columns - 1) as f32;
                let v = y as f32 / (rows - 1) as f32;
                control_points.push(SurfaceControlPoint {
                    parameter: Point2::new(u, v),
                    source: Point2::new(
                        u * (source_width - 1) as f32,
                        v * (source_height - 1) as f32,
                    ),
                    surface: Point3 { x: u, y: v, z: 0.0 },
                });
            }
        }
        let grid = Self {
            columns,
            rows,
            control_points,
        };
        grid.validate()?;
        Ok(grid)
    }
}

/// Coordinate parameterization selected by a surface reconstruction family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceParameterization {
    /// Regular UV grid suitable for a dense inverse field.
    RegularUv,
    /// Future generalized cylindrical surface for document dewarping.
    GeneralizedCylinder,
    /// Future domain-provided parameterization.
    DomainDefined,
}

/// Mesh-like warp expressed by a validated regular control grid.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshWarp {
    /// Control-grid geometry.
    pub grid: SurfaceGrid,
}

/// Generic surface model that a domain may use for reconstruction.
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceModel {
    /// Surface coordinate convention.
    pub parameterization: SurfaceParameterization,
    /// Control mesh.
    pub mesh: MeshWarp,
}

/// Conservative curvature observation and an explicit evidence boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct CurvatureEstimate {
    /// Estimated curvature likelihood, not a calibrated physical curvature.
    pub likelihood: f32,
    /// Confidence in the available evidence.
    pub confidence: Confidence,
    /// Evidence or placeholder limitations.
    pub diagnostics: Vec<String>,
}

impl CurvatureEstimate {
    /// Creates the MP4 no-evidence placeholder instead of inventing curvature.
    pub fn unavailable() -> Self {
        Self {
            likelihood: 0.0,
            confidence: Confidence::default(),
            diagnostics: vec![
                "curvature_estimation_not_implemented_without_line_or_surface_evidence".into(),
            ],
        }
    }
}

/// Output of a surface reconstruction attempt.
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceReconstructionResult {
    /// Optional estimated surface model.
    pub surface: Option<SurfaceModel>,
    /// Optional validated inverse warp field.
    pub dense_warp: Option<DenseWarpField>,
    /// Curvature evidence carried to condition routing.
    pub curvature: CurvatureEstimate,
    /// Explicit implementation and routing diagnostics.
    pub diagnostics: Vec<String>,
}

/// Document-facing curvature classification used to select a safe reconstruction path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurvedPageClass {
    /// Geometry is suitable for a planar homography.
    FlatPage,
    /// Surface reconstruction is indicated but not implemented by MP4.
    MildCurvature,
    /// Surface reconstruction is required; planar output would misrepresent the page.
    StrongCurvature,
    /// Available evidence cannot classify curvature.
    Unknown,
}

/// Safe route chosen for an observed curved-page class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CurvedPageRoute {
    /// Use the implemented planar path.
    Planar,
    /// Surface processing is required but unavailable, so callers must not hide it.
    SurfaceUnavailable { diagnostic: String },
}

/// Selects a safe route without silently flattening a detected curved page.
pub fn route_curved_page(classification: CurvedPageClass) -> CurvedPageRoute {
    match classification {
        CurvedPageClass::FlatPage => CurvedPageRoute::Planar,
        CurvedPageClass::MildCurvature | CurvedPageClass::StrongCurvature => {
            CurvedPageRoute::SurfaceUnavailable {
                diagnostic: "curved_page_requires_surface_reconstruction_not_available_in_mp4"
                    .into(),
            }
        }
        CurvedPageClass::Unknown => CurvedPageRoute::SurfaceUnavailable {
            diagnostic: "curvature_unknown_planar_reconstruction_requires_explicit_caller_policy"
                .into(),
        },
    }
}

/// Converts a validated regular mesh to a dense output-to-source coordinate field.
pub fn mesh_to_dense_warp(
    mesh: &MeshWarp,
    output_width: u32,
    output_height: u32,
) -> PerceptionResult<DenseWarpField> {
    mesh.grid.validate()?;
    if output_width == 0 || output_height == 0 {
        return Err(PerceptionError::InvalidDimensions {
            width: output_width,
            height: output_height,
        });
    }
    let mut vectors = Vec::with_capacity((output_width as usize) * (output_height as usize));
    for y in 0..output_height {
        let v = if output_height == 1 {
            0.0
        } else {
            y as f32 / (output_height - 1) as f32
        };
        for x in 0..output_width {
            let u = if output_width == 1 {
                0.0
            } else {
                x as f32 / (output_width - 1) as f32
            };
            vectors.push(sample_mesh(mesh, u, v));
        }
    }
    Ok(DenseWarpField {
        width: output_width,
        height: output_height,
        vectors,
    })
}

/// Applies a dense inverse warp through the checked core remap implementation.
pub fn apply_dense_warp(
    input: &ImageBuffer,
    field: &DenseWarpField,
) -> PerceptionResult<ImageBuffer> {
    remap(input, field, SamplingMode::Bilinear, WarpBorder::Replicate)
}

fn sample_mesh(mesh: &MeshWarp, u: f32, v: f32) -> Point2 {
    let columns = mesh.grid.columns as usize;
    let rows = mesh.grid.rows as usize;
    let x = u.clamp(0.0, 1.0) * (columns - 1) as f32;
    let y = v.clamp(0.0, 1.0) * (rows - 1) as f32;
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(columns - 1);
    let y1 = (y0 + 1).min(rows - 1);
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let p00 = mesh.grid.control_points[y0 * columns + x0].source;
    let p10 = mesh.grid.control_points[y0 * columns + x1].source;
    let p01 = mesh.grid.control_points[y1 * columns + x0].source;
    let p11 = mesh.grid.control_points[y1 * columns + x1].source;
    let top = Point2::new(p00.x + (p10.x - p00.x) * tx, p00.y + (p10.y - p00.y) * tx);
    let bottom = Point2::new(p01.x + (p11.x - p01.x) * tx, p01.y + (p11.y - p01.y) * tx);
    Point2::new(
        top.x + (bottom.x - top.x) * ty,
        top.y + (bottom.y - top.y) * ty,
    )
}
