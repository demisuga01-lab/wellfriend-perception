//! Deterministic planar canonical-page reconstruction for the document domain.

use wellfriend_perception_core::{
    Confidence, ImageBuffer, ImageShape, PerceptionError, PerceptionResult, Point2, Quad, Score,
    Transform2D,
    geometry::{SamplingMode, WarpBorder, estimate_homography_4pt, warp_perspective},
};

use crate::{
    CanonicalGeometry, CanonicalRepresentation, LensCorrectionModel, NoOpLensCorrector,
    ReconstructionArtifact, ReconstructionConfidence, ReconstructionContext,
    ReconstructionDiagnostics, ReconstructionFamily, ReconstructionOutput, ReconstructionQuality,
    ReconstructionStage, Reconstructor,
};

/// Aspect policy applied before page-size selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AspectRatioPolicy {
    /// Derive the output aspect from opposite quad edges.
    FreeFromQuad,
    /// Use a known paper/card preset; no automatic classifier is implied.
    KnownPreset(PaperPreset),
    /// Explicit future seam for a model or document-class decision.
    DetectedDocumentClassPlaceholder,
    /// Caller supplies a physical width-to-height ratio.
    ManualOverride { width: f32, height: f32 },
    /// Use a declared preset only if free-aspect inference fails.
    Fallback(PaperPreset),
}

/// Known page and card proportions available without automatic classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaperPreset {
    /// ISO 216 A4, 210 x 297 mm.
    A4,
    /// ISO 216 A5, 148 x 210 mm.
    A5,
    /// North American Letter, 8.5 x 11 in.
    Letter,
    /// North American Legal, 8.5 x 14 in.
    Legal,
    /// Preserve a receipt's inferred free aspect.
    ReceiptFree,
    /// ISO/IEC 7810 ID-1 card, 85.60 x 53.98 mm.
    IDCard,
    /// Conventional business-card reference, 85 x 55 mm.
    BusinessCard,
    /// Caller-defined physical ratio.
    Custom { width: u32, height: u32 },
}

/// Orientation policy applied while choosing canonical page axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrientationPolicy {
    /// Keep the source quad corner ordering.
    PreserveSource,
    /// Ensure canonical height is at least canonical width.
    LongEdgeVertical,
    /// Ensure canonical width is at least canonical height.
    LongEdgeHorizontal,
    /// Rotate canonical corner correspondence by 0, 90, 180, or 270 degrees.
    ManualRotation { degrees_clockwise: u16 },
    /// Explicit seam for future image metadata orientation.
    MetadataDriven,
}

/// Boundary treatment around the canonical page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CropMarginPolicy {
    /// Map the source quad exactly to the output border.
    None,
    /// Keep one safe inner pixel around the mapped page.
    SafeInner,
    /// Include the mapped page border at the output edge.
    IncludeBorder,
    /// Add a bounded percentage of output pixels around the mapped page.
    ExpandPercent(f32),
    /// Add explicit output-side margins in canonical pixels.
    Manual {
        /// Left margin.
        left: u32,
        /// Top margin.
        top: u32,
        /// Right margin.
        right: u32,
        /// Bottom margin.
        bottom: u32,
    },
}

/// Declared page-coordinate axes for later semantic and export layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageCoordinateSystem {
    /// Pixel origin at top left; x increases right and y increases down.
    PixelsTopLeft,
    /// Physical coordinate information is known or intentionally declared.
    PhysicalMillimeters,
}

/// Physical page dimensions when a preset or manual policy supplies them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PagePhysicalSize {
    /// Physical dimensions are not known from geometry alone.
    Unknown,
    /// Approximate known physical width and height in millimeters.
    Millimeters { width: f32, height: f32 },
    /// Caller provides a ratio without an absolute scale.
    AspectOnly { width_over_height: f32 },
}

/// Canonical page geometry and policy decisions.
#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalPageGeometry {
    /// Canonical coordinate convention.
    pub coordinate_system: PageCoordinateSystem,
    /// Output width.
    pub width: u32,
    /// Output height.
    pub height: u32,
    /// Inferred or declared width-to-height aspect.
    pub aspect_ratio: f32,
    /// Available physical-size declaration.
    pub physical_size: PagePhysicalSize,
    /// Orientation policy that formed the final axes.
    pub orientation: OrientationPolicy,
    /// Margin policy selected by the caller.
    pub crop_margin: CropMarginPolicy,
}

/// Ordered transforms applied to a canonical page.
#[derive(Clone, Debug, PartialEq)]
pub struct PageTransformChain {
    /// Refined/fused source quad after validation and optional orientation rotation.
    pub source_quad: Quad,
    /// Homography from source image to canonical output pixels.
    pub source_to_page: Transform2D,
    /// Lens model requested at the explicit no-op seam.
    pub lens_model: LensCorrectionModel,
    /// Resampling implementation selected by policy.
    pub resampling: SamplingMode,
}

/// Trace information specific to one reconstructed page.
#[derive(Clone, Debug, PartialEq)]
pub struct PageReconstructionTrace {
    /// Source image dimensions before reconstruction.
    pub source_size: ImageShape,
    /// Selected transform chain.
    pub transform_chain: PageTransformChain,
    /// Ordered reconstruction diagnostics.
    pub diagnostics: Vec<String>,
}

/// Quality information carried after the canonical page has been generated.
#[derive(Clone, Debug, PartialEq)]
pub struct PageQualityAfterReconstruction {
    /// Shared reconstruction quality summary.
    pub summary: ReconstructionQuality,
    /// Output blur score, where higher is sharper under the scalar analyzer.
    pub output_blur_score: Score,
    /// Output exposure score, where higher is closer to balanced exposure.
    pub output_exposure_score: Score,
    /// Output contrast score, where higher is more contrasty under the scalar analyzer.
    pub output_contrast_score: Score,
    /// Risk that a policy or mapped border clips desired source content.
    pub border_clipping_risk: Score,
}

/// Canonical page image and all geometry needed by future semantics/export layers.
#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalPage {
    /// Materialized canonical page pixels.
    pub image: ImageBuffer,
    /// Canonical geometry and policies.
    pub geometry: CanonicalPageGeometry,
    /// Source-to-page transform history.
    pub trace: PageReconstructionTrace,
    /// Quality after projective resampling.
    pub quality: PageQualityAfterReconstruction,
    /// Bounded implementation confidence.
    pub confidence: ReconstructionConfidence,
}

/// Document canonical representation; OCR is intentionally absent at this stage.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanonicalDocument {
    /// Canonical pages in scan/session order.
    pub pages: Vec<CanonicalPage>,
    /// Domain-level diagnostics not tied to an individual page.
    pub diagnostics: Vec<String>,
}

/// Input for the real MP4 planar document reconstructor.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarDocumentInput {
    /// Original image that contains the selected page quad.
    pub image: ImageBuffer,
    /// Fused/refined document quad in source-image coordinates.
    pub quad: Quad,
}

/// Deterministic scalar reconstruction tuning.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanarReconstructionConfig {
    /// Target pixel length of the output's long edge before margin expansion.
    pub target_long_edge: u32,
    /// Hard maximum to prevent accidental oversized buffers.
    pub maximum_dimension: u32,
    /// Aspect decision policy.
    pub aspect_policy: AspectRatioPolicy,
    /// Orientation policy.
    pub orientation_policy: OrientationPolicy,
    /// Output margin policy.
    pub crop_margin_policy: CropMarginPolicy,
    /// Scalar sampling implementation.
    pub sampling: SamplingMode,
    /// Outside-source behavior during mapping.
    pub border: WarpBorder,
    /// Lens correction seam configuration.
    pub lens_model: LensCorrectionModel,
}

impl Default for PlanarReconstructionConfig {
    fn default() -> Self {
        Self {
            target_long_edge: 1600,
            maximum_dimension: 4096,
            aspect_policy: AspectRatioPolicy::FreeFromQuad,
            orientation_policy: OrientationPolicy::PreserveSource,
            crop_margin_policy: CropMarginPolicy::IncludeBorder,
            sampling: SamplingMode::Bilinear,
            border: WarpBorder::Replicate,
            lens_model: LensCorrectionModel::None,
        }
    }
}

/// Scalar planar document reconstructor based on the checked MP2 homography/warp path.
#[derive(Clone, Debug, Default)]
pub struct PlanarDocumentReconstructor {
    /// Caller-visible deterministic configuration.
    pub config: PlanarReconstructionConfig,
}

impl PlanarDocumentReconstructor {
    /// Reconstructs one page and wraps it in a canonical document.
    pub fn reconstruct_page(&self, input: &PlanarDocumentInput) -> PerceptionResult<CanonicalPage> {
        input.quad.validate()?;
        self.validate_config()?;
        let (lens_corrected, lens_stage) =
            NoOpLensCorrector.apply(&input.image, &self.config.lens_model)?;
        let source_quad = oriented_quad(input.quad, self.config.orientation_policy)?;
        let inferred_aspect = infer_page_aspect(source_quad)?;
        let (base_aspect, physical_size, aspect_diagnostic) =
            resolve_aspect(self.config.aspect_policy, inferred_aspect)?;
        let (base_width, base_height, final_aspect) = output_dimensions(
            base_aspect,
            self.config.target_long_edge,
            self.config.maximum_dimension,
            self.config.orientation_policy,
        )?;
        let margins = resolve_margins(self.config.crop_margin_policy, base_width, base_height)?;
        let output_width = base_width
            .checked_add(margins.0)
            .and_then(|value| value.checked_add(margins.2))
            .ok_or(PerceptionError::Overflow)?;
        let output_height = base_height
            .checked_add(margins.1)
            .and_then(|value| value.checked_add(margins.3))
            .ok_or(PerceptionError::Overflow)?;
        if output_width > self.config.maximum_dimension
            || output_height > self.config.maximum_dimension
        {
            return Err(PerceptionError::InvalidDimensions {
                width: output_width,
                height: output_height,
            });
        }
        let target = [
            Point2::new(margins.0 as f32, margins.1 as f32),
            Point2::new((margins.0 + base_width - 1) as f32, margins.1 as f32),
            Point2::new(
                (margins.0 + base_width - 1) as f32,
                (margins.1 + base_height - 1) as f32,
            ),
            Point2::new(margins.0 as f32, (margins.1 + base_height - 1) as f32),
        ];
        let source_to_page = estimate_homography_4pt(source_quad.points, target)?;
        let page_image = warp_perspective(
            &lens_corrected,
            source_to_page,
            ImageShape::new(output_width, output_height)?,
            self.config.sampling,
            self.config.border,
        )?;
        let reconstruction_quality = crate::quality::evaluate_reconstruction_quality(
            &page_image,
            source_quad,
            output_width,
            output_height,
            margins,
            final_aspect,
        )?;
        let quality = PageQualityAfterReconstruction {
            output_blur_score: reconstruction_quality.output_blur_score,
            output_exposure_score: reconstruction_quality.output_exposure_score,
            output_contrast_score: reconstruction_quality.output_contrast_score,
            border_clipping_risk: reconstruction_quality.border_clipping_risk,
            summary: reconstruction_quality.summary,
        };
        let confidence = ReconstructionConfidence {
            value: Confidence::new((1.0 - quality.summary.warp_stretch_risk.value() * 0.35).clamp(0.0, 1.0))?,
            diagnostics: vec![
                "planar confidence is a deterministic geometry heuristic, not calibrated probability".into(),
            ],
        };
        Ok(CanonicalPage {
            image: page_image,
            geometry: CanonicalPageGeometry {
                coordinate_system: PageCoordinateSystem::PixelsTopLeft,
                width: output_width,
                height: output_height,
                aspect_ratio: final_aspect,
                physical_size,
                orientation: self.config.orientation_policy,
                crop_margin: self.config.crop_margin_policy,
            },
            trace: PageReconstructionTrace {
                source_size: input.image.shape(),
                transform_chain: PageTransformChain {
                    source_quad,
                    source_to_page,
                    lens_model: self.config.lens_model.clone(),
                    resampling: self.config.sampling,
                },
                diagnostics: vec![
                    "validated fused/refined quad before homography estimation".into(),
                    lens_stage.diagnostics.join(","),
                    aspect_diagnostic,
                    format!(
                        "canonical margins left={} top={} right={} bottom={}",
                        margins.0, margins.1, margins.2, margins.3
                    ),
                ],
            },
            quality,
            confidence,
        })
    }

    fn validate_config(&self) -> PerceptionResult<()> {
        if self.config.target_long_edge < 2
            || self.config.maximum_dimension < self.config.target_long_edge
        {
            return Err(PerceptionError::InvalidDimensions {
                width: self.config.target_long_edge,
                height: self.config.maximum_dimension,
            });
        }
        if let CropMarginPolicy::ExpandPercent(percent) = self.config.crop_margin_policy
            && (!percent.is_finite() || !(0.0..=0.45).contains(&percent))
        {
            return Err(PerceptionError::NumericFailure {
                reason: "expand margin percent must be finite and in [0, 0.45]".into(),
            });
        }
        if let OrientationPolicy::ManualRotation { degrees_clockwise } =
            self.config.orientation_policy
            && !matches!(degrees_clockwise, 0 | 90 | 180 | 270)
        {
            return Err(PerceptionError::UnsupportedOperation {
                operation: "manual orientation supports only 0, 90, 180, or 270 degrees",
            });
        }
        Ok(())
    }
}

impl Reconstructor for PlanarDocumentReconstructor {
    type Input = PlanarDocumentInput;
    type Output = CanonicalDocument;

    fn reconstruct(
        &self,
        input: &Self::Input,
        context: &ReconstructionContext,
    ) -> PerceptionResult<Self::Output> {
        let page = self.reconstruct_page(input)?;
        let mut diagnostics = vec!["document planar reconstruction completed".into()];
        if let Some(domain_id) = &context.domain_id {
            diagnostics.push(format!("reconstruction context domain={domain_id}"));
        }
        Ok(CanonicalDocument {
            pages: vec![page],
            diagnostics,
        })
    }
}

impl PlanarDocumentReconstructor {
    /// Returns the generic canonical output envelope for a reconstructed page.
    pub fn reconstruct_output(
        &self,
        input: &PlanarDocumentInput,
    ) -> PerceptionResult<ReconstructionOutput> {
        let page = self.reconstruct_page(input)?;
        let diagnostics = ReconstructionDiagnostics {
            stages: vec![
                ReconstructionStage::InputValidation,
                ReconstructionStage::LensCorrection,
                ReconstructionStage::PlanarWarp,
                ReconstructionStage::QualityEvaluation,
            ],
            messages: page.trace.diagnostics.clone(),
            attributes: Default::default(),
        };
        Ok(ReconstructionOutput {
            canonical: CanonicalRepresentation {
                family: ReconstructionFamily::Planar,
                geometry: CanonicalGeometry::Planar {
                    width: page.geometry.width,
                    height: page.geometry.height,
                    origin: Point2::new(0.0, 0.0),
                },
                artifacts: vec![ReconstructionArtifact::Image(page.image.clone())],
                diagnostics: diagnostics.clone(),
            },
            confidence: page.confidence,
            quality: page.quality.summary,
            diagnostics,
        })
    }
}

/// Infers a free page aspect from averaged opposite quad edges.
pub fn infer_page_aspect(quad: Quad) -> PerceptionResult<f32> {
    quad.validate()?;
    let top = quad.points[0].distance(quad.points[1]);
    let right = quad.points[1].distance(quad.points[2]);
    let bottom = quad.points[2].distance(quad.points[3]);
    let left = quad.points[3].distance(quad.points[0]);
    let width = (top + bottom) * 0.5;
    let height = (left + right) * 0.5;
    if !width.is_finite() || !height.is_finite() || height <= f32::EPSILON {
        return Err(PerceptionError::DegenerateGeometry {
            reason: "page aspect cannot be inferred from zero-length edges".into(),
        });
    }
    Ok(width / height)
}

fn resolve_aspect(
    policy: AspectRatioPolicy,
    inferred: f32,
) -> PerceptionResult<(f32, PagePhysicalSize, String)> {
    if !inferred.is_finite() || inferred <= 0.0 {
        return Err(PerceptionError::DegenerateGeometry {
            reason: "inferred aspect must be finite and positive".into(),
        });
    }
    match policy {
        AspectRatioPolicy::FreeFromQuad => Ok((
            inferred,
            PagePhysicalSize::AspectOnly {
                width_over_height: inferred,
            },
            "aspect_policy=free_from_quad".into(),
        )),
        AspectRatioPolicy::KnownPreset(preset) => preset_aspect(preset),
        AspectRatioPolicy::DetectedDocumentClassPlaceholder => Ok((
            inferred,
            PagePhysicalSize::AspectOnly {
                width_over_height: inferred,
            },
            "aspect_policy=detected_document_class_placeholder_fell_back_to_quad".into(),
        )),
        AspectRatioPolicy::ManualOverride { width, height } => {
            if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
                return Err(PerceptionError::InvalidDimensions {
                    width: width.max(0.0) as u32,
                    height: height.max(0.0) as u32,
                });
            }
            Ok((
                width / height,
                PagePhysicalSize::AspectOnly {
                    width_over_height: width / height,
                },
                "aspect_policy=manual_override".into(),
            ))
        }
        AspectRatioPolicy::Fallback(preset) => {
            let aspect = if (0.1..=10.0).contains(&inferred) {
                inferred
            } else {
                return preset_aspect(preset);
            };
            Ok((
                aspect,
                PagePhysicalSize::AspectOnly {
                    width_over_height: aspect,
                },
                "aspect_policy=fallback_used_free_quad".into(),
            ))
        }
    }
}

fn preset_aspect(preset: PaperPreset) -> PerceptionResult<(f32, PagePhysicalSize, String)> {
    let (width, height, name) = match preset {
        PaperPreset::A4 => (210.0, 297.0, "A4"),
        PaperPreset::A5 => (148.0, 210.0, "A5"),
        PaperPreset::Letter => (215.9, 279.4, "Letter"),
        PaperPreset::Legal => (215.9, 355.6, "Legal"),
        PaperPreset::IDCard => (85.60, 53.98, "IDCard"),
        PaperPreset::BusinessCard => (85.0, 55.0, "BusinessCard"),
        PaperPreset::ReceiptFree => {
            return Err(PerceptionError::UnsupportedOperation {
                operation: "ReceiptFree requires free aspect from a quad",
            });
        }
        PaperPreset::Custom { width, height } => (width as f32, height as f32, "Custom"),
    };
    if width <= 0.0 || height <= 0.0 {
        return Err(PerceptionError::InvalidDimensions {
            width: width as u32,
            height: height as u32,
        });
    }
    Ok((
        width / height,
        PagePhysicalSize::Millimeters { width, height },
        format!("aspect_policy=known_preset:{name}"),
    ))
}

fn output_dimensions(
    aspect: f32,
    long_edge: u32,
    maximum: u32,
    orientation: OrientationPolicy,
) -> PerceptionResult<(u32, u32, f32)> {
    if !aspect.is_finite() || aspect <= 0.0 {
        return Err(PerceptionError::DegenerateGeometry {
            reason: "output aspect must be finite and positive".into(),
        });
    }
    let mut final_aspect = aspect;
    if matches!(orientation, OrientationPolicy::LongEdgeVertical) && final_aspect > 1.0 {
        final_aspect = 1.0 / final_aspect;
    }
    if matches!(orientation, OrientationPolicy::LongEdgeHorizontal) && final_aspect < 1.0 {
        final_aspect = 1.0 / final_aspect;
    }
    let (width, height) = if final_aspect >= 1.0 {
        (long_edge, (long_edge as f32 / final_aspect).round() as u32)
    } else {
        ((long_edge as f32 * final_aspect).round() as u32, long_edge)
    };
    if width < 2 || height < 2 || width > maximum || height > maximum {
        return Err(PerceptionError::InvalidDimensions { width, height });
    }
    Ok((width, height, width as f32 / height as f32))
}

fn resolve_margins(
    policy: CropMarginPolicy,
    width: u32,
    height: u32,
) -> PerceptionResult<(u32, u32, u32, u32)> {
    let margins = match policy {
        CropMarginPolicy::None | CropMarginPolicy::IncludeBorder => (0, 0, 0, 0),
        CropMarginPolicy::SafeInner => (1, 1, 1, 1),
        CropMarginPolicy::ExpandPercent(percent) => {
            let x = (width as f32 * percent).round() as u32;
            let y = (height as f32 * percent).round() as u32;
            (x, y, x, y)
        }
        CropMarginPolicy::Manual {
            left,
            top,
            right,
            bottom,
        } => (left, top, right, bottom),
    };
    let inner_width = width
        .checked_add(margins.0)
        .and_then(|value| value.checked_add(margins.2))
        .ok_or(PerceptionError::Overflow)?;
    let inner_height = height
        .checked_add(margins.1)
        .and_then(|value| value.checked_add(margins.3))
        .ok_or(PerceptionError::Overflow)?;
    if inner_width < 2 || inner_height < 2 {
        return Err(PerceptionError::InvalidDimensions {
            width: inner_width,
            height: inner_height,
        });
    }
    Ok(margins)
}

fn oriented_quad(quad: Quad, orientation: OrientationPolicy) -> PerceptionResult<Quad> {
    quad.validate()?;
    let shifts = match orientation {
        OrientationPolicy::ManualRotation { degrees_clockwise } => match degrees_clockwise {
            0 => 0,
            90 => 3,
            180 => 2,
            270 => 1,
            _ => unreachable!("validated by configuration"),
        },
        _ => 0,
    };
    Ok(Quad {
        points: std::array::from_fn(|index| quad.points[(index + shifts) % 4]),
    })
}
