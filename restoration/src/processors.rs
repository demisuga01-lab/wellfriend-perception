//! Scalar restoration processors and model-ready processor contracts.

use wellfriend_perception_core::{ImageBuffer, PerceptionResult, PixelFormat};
use wellfriend_perception_image::{
    adaptive_mean_threshold, gamma_gray, gaussian_blur_gray, grayscale, median_blur_3x3,
    min_max_normalize_gray, otsu_threshold, threshold_gray, unsharp_mask_gray,
};

use crate::{ConditionVector, DeviceClass, ProcessorCapability, ProcessorCost, ProcessorId};

/// Image and metadata supplied to one restoration processor.
#[derive(Clone, Debug, PartialEq)]
pub struct RestorationInput {
    /// Canonical or original image chosen by a plan.
    pub image: ImageBuffer,
}

/// Runtime information that affects processor selection, not image semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessingContext {
    /// Device budget profile.
    pub device_class: DeviceClass,
    /// Whether the caller requested full-quality rather than preview work.
    pub full_quality: bool,
}

impl Default for ProcessingContext {
    fn default() -> Self {
        Self {
            device_class: DeviceClass::Unknown,
            full_quality: true,
        }
    }
}

/// Predicted cost and benefit before executing a processor.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessingEstimate {
    /// Predicted quality benefit as a bounded scalar.
    pub expected_benefit: f32,
    /// Relative scalar cost.
    pub estimated_cost: ProcessorCost,
    /// Implementation notes.
    pub diagnostics: Vec<String>,
}

/// Result of a restoration processor.
#[derive(Clone, Debug, PartialEq)]
pub struct RestorationOutput {
    /// Transformed image.
    pub image: ImageBuffer,
    /// Ordered applied processor ids.
    pub applied_processors: Vec<ProcessorId>,
    /// Traceable implementation diagnostics.
    pub diagnostics: Vec<String>,
}

/// A processor that can be selected and executed by a restoration plan.
pub trait RestorationProcessor {
    /// Stable processor id.
    fn id(&self) -> ProcessorId;
    /// Declared processor capabilities.
    fn capabilities(&self) -> &[ProcessorCapability];
    /// Predicts cost/benefit without changing the image.
    fn estimate(
        &self,
        input: &RestorationInput,
        conditions: &ConditionVector,
    ) -> ProcessingEstimate;
    /// Performs a checked scalar or future model-backed transformation.
    fn process(
        &self,
        input: &RestorationInput,
        context: &ProcessingContext,
    ) -> PerceptionResult<RestorationOutput>;
}

/// Threshold implementation selected by the scalar binarization processor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinarizationMode {
    /// Use a stable caller-provided threshold.
    Fixed(u8),
    /// Estimate a global Otsu threshold.
    Otsu,
    /// Use local mean thresholding with a fixed odd window.
    AdaptiveMean { window: u32, offset: f32 },
}

/// All MP4 scalar processors without model runtime dependencies.
#[derive(Clone, Debug, PartialEq)]
pub enum ScalarRestorationProcessor {
    /// Global min/max normalization of a gray image.
    BrightnessContrast,
    /// Positive gamma correction after grayscale conversion.
    Gamma { gamma: f32 },
    /// Median or Gaussian scalar denoise.
    Denoise { median: bool },
    /// Scalar high-frequency enhancement.
    Unsharp { amount: f32 },
    /// Slow illumination baseline using min/max normalization.
    BackgroundNormalization,
    /// Standard grayscale conversion.
    Grayscale,
    /// Fixed, Otsu, or adaptive-mean binary threshold.
    Binarize { mode: BinarizationMode },
}

impl ScalarRestorationProcessor {
    /// Default processor for a stable id used by the router.
    pub fn from_id(id: &ProcessorId) -> Option<Self> {
        match id.as_str() {
            "brightness_contrast" => Some(Self::BrightnessContrast),
            "gamma" => Some(Self::Gamma { gamma: 0.9 }),
            "denoise" => Some(Self::Denoise { median: true }),
            "unsharp" => Some(Self::Unsharp { amount: 0.7 }),
            "background_normalization" => Some(Self::BackgroundNormalization),
            "grayscale" => Some(Self::Grayscale),
            "binarize" => Some(Self::Binarize {
                mode: BinarizationMode::Otsu,
            }),
            _ => None,
        }
    }

    fn gray_input(&self, image: &ImageBuffer) -> PerceptionResult<ImageBuffer> {
        grayscale(image)
    }
}

impl RestorationProcessor for ScalarRestorationProcessor {
    fn id(&self) -> ProcessorId {
        match self {
            Self::BrightnessContrast => ProcessorId::new("brightness_contrast"),
            Self::Gamma { .. } => ProcessorId::new("gamma"),
            Self::Denoise { .. } => ProcessorId::new("denoise"),
            Self::Unsharp { .. } => ProcessorId::new("unsharp"),
            Self::BackgroundNormalization => ProcessorId::new("background_normalization"),
            Self::Grayscale => ProcessorId::new("grayscale"),
            Self::Binarize { .. } => ProcessorId::new("binarize"),
        }
    }

    fn capabilities(&self) -> &[ProcessorCapability] {
        match self {
            Self::BrightnessContrast => &[ProcessorCapability::BrightnessContrast],
            Self::Gamma { .. } => &[ProcessorCapability::Gamma],
            Self::Denoise { .. } => &[ProcessorCapability::Denoise],
            Self::Unsharp { .. } => &[ProcessorCapability::Sharpen],
            Self::BackgroundNormalization => &[ProcessorCapability::BackgroundNormalization],
            Self::Grayscale => &[ProcessorCapability::Grayscale],
            Self::Binarize { .. } => &[ProcessorCapability::Binarization],
        }
    }

    fn estimate(
        &self,
        _input: &RestorationInput,
        conditions: &ConditionVector,
    ) -> ProcessingEstimate {
        let (benefit, cost) = match self {
            Self::BrightnessContrast => (
                conditions
                    .score(crate::ConditionKind::LowContrast)
                    .max(conditions.score(crate::ConditionKind::Underexposure)),
                1,
            ),
            Self::Gamma { .. } => (conditions.score(crate::ConditionKind::Underexposure), 1),
            Self::Denoise { .. } => (conditions.score(crate::ConditionKind::Noise), 2),
            Self::Unsharp { .. } => (conditions.score(crate::ConditionKind::Blur), 2),
            Self::BackgroundNormalization => (
                conditions
                    .score(crate::ConditionKind::Shadow)
                    .max(conditions.score(crate::ConditionKind::LowContrast)),
                2,
            ),
            Self::Grayscale => (0.5, 1),
            Self::Binarize { .. } => (conditions.score(crate::ConditionKind::LowContrast), 2),
        };
        ProcessingEstimate {
            expected_benefit: benefit,
            estimated_cost: ProcessorCost {
                latency_units: cost,
                memory_units: cost,
            },
            diagnostics: vec!["scalar estimate is heuristic and uncalibrated".into()],
        }
    }

    fn process(
        &self,
        input: &RestorationInput,
        _context: &ProcessingContext,
    ) -> PerceptionResult<RestorationOutput> {
        let image = match self {
            Self::Grayscale => grayscale(&input.image)?,
            Self::BrightnessContrast | Self::BackgroundNormalization => {
                min_max_normalize_gray(&self.gray_input(&input.image)?)?
            }
            Self::Gamma { gamma } => gamma_gray(&self.gray_input(&input.image)?, *gamma)?,
            Self::Denoise { median } => {
                let gray = self.gray_input(&input.image)?;
                if *median {
                    median_blur_3x3(&gray)?
                } else {
                    gaussian_blur_gray(&gray)?
                }
            }
            Self::Unsharp { amount } => {
                unsharp_mask_gray(&self.gray_input(&input.image)?, *amount)?
            }
            Self::Binarize { mode } => {
                let gray = self.gray_input(&input.image)?;
                match mode {
                    BinarizationMode::Fixed(threshold) => threshold_gray(&gray, *threshold)?,
                    BinarizationMode::Otsu => threshold_gray(&gray, otsu_threshold(&gray)?)?,
                    BinarizationMode::AdaptiveMean { window, offset } => {
                        adaptive_mean_threshold(&gray, *window, *offset)?
                    }
                }
            }
        };
        debug_assert_eq!(image.pixel_format(), PixelFormat::Gray8);
        Ok(RestorationOutput {
            image,
            applied_processors: vec![self.id()],
            diagnostics: vec![format!("applied scalar processor {}", self.id().as_str())],
        })
    }
}
