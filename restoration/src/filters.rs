//! Document filter presets expressed as explicit and safe processing plans.

use wellfriend_perception_core::{ImageBuffer, PerceptionResult};

use crate::{
    ConditionVector, DeviceClass, ProcessingContext, RestorationInput, RestorationOutput,
    RestorationProcessor, ScalarRestorationProcessor, SpecialistRouter,
};

/// Named document filter goals.  Presets are plans, not undocumented image mutations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DocumentFilterPreset {
    /// Preserve source image pixels exactly.
    Original,
    /// Route scalar processors from available conditions.
    Auto,
    /// Conservative cleanup for text-forward documents.
    Clean,
    /// Keep color; only permitted color-safe scalar changes may run.
    Color,
    /// Convert to Gray8.
    Grayscale,
    /// Convert to binary Gray8 using a documented threshold.
    BlackAndWhite,
    /// Declared future receipt plan; MP4 does not claim specialized behavior.
    Receipt,
    /// Declared future book/dewarp plan; MP4 does not claim specialized behavior.
    Book,
    /// Declared future whiteboard plan; MP4 does not claim specialized behavior.
    Whiteboard,
    /// Declared future photo-document safety plan.
    PhotoDocument,
}

/// Static preset contract available to applications before processing starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterPlanContract {
    /// User-independent processing goal.
    pub goal: &'static str,
    /// Processor identifiers allowed by this preset.
    pub allowed_processors: Vec<&'static str>,
    /// Processor identifiers forbidden by this preset.
    pub forbidden_processors: Vec<&'static str>,
    /// Default safe order.
    pub default_order: Vec<&'static str>,
    /// Safety constraints that must remain visible to callers.
    pub safety_constraints: Vec<&'static str>,
    /// Relative preview cost classification.
    pub preview_cost_level: &'static str,
    /// Relative full-resolution cost classification.
    pub full_quality_cost_level: &'static str,
}

/// Document filter graph that delegates selection to the deterministic specialist router.
#[derive(Clone, Debug, Default)]
pub struct DocumentFilterGraph {
    /// Router used for `Auto` and baseline safe plans.
    pub router: SpecialistRouter,
}

impl DocumentFilterGraph {
    /// Returns the stable contract for a document filter preset.
    pub fn contract(preset: DocumentFilterPreset) -> FilterPlanContract {
        match preset {
            DocumentFilterPreset::Original => FilterPlanContract {
                goal: "preserve source pixels",
                allowed_processors: Vec::new(),
                forbidden_processors: vec!["*"],
                default_order: Vec::new(),
                safety_constraints: vec!["no pixels may change"],
                preview_cost_level: "none",
                full_quality_cost_level: "none",
            },
            DocumentFilterPreset::Auto => FilterPlanContract {
                goal: "apply only condition-selected scalar restoration",
                allowed_processors: vec![
                    "grayscale",
                    "brightness_contrast",
                    "gamma",
                    "denoise",
                    "background_normalization",
                    "unsharp",
                    "binarize",
                ],
                forbidden_processors: vec!["neural_restoration"],
                default_order: vec![
                    "grayscale",
                    "brightness_contrast",
                    "denoise",
                    "background_normalization",
                    "unsharp",
                    "binarize",
                ],
                safety_constraints: vec![
                    "conflicting processors cannot co-exist",
                    "selection must remain explainable",
                ],
                preview_cost_level: "low",
                full_quality_cost_level: "medium",
            },
            DocumentFilterPreset::Clean => FilterPlanContract {
                goal: "conservative text-forward cleanup",
                allowed_processors: vec![
                    "grayscale",
                    "denoise",
                    "background_normalization",
                    "unsharp",
                ],
                forbidden_processors: vec!["binarize", "neural_restoration"],
                default_order: vec![
                    "grayscale",
                    "denoise",
                    "background_normalization",
                    "unsharp",
                ],
                safety_constraints: vec!["preserve thin handwriting where evidence is absent"],
                preview_cost_level: "low",
                full_quality_cost_level: "medium",
            },
            DocumentFilterPreset::Color => FilterPlanContract {
                goal: "retain color appearance",
                allowed_processors: vec!["brightness_contrast", "gamma", "denoise"],
                forbidden_processors: vec!["grayscale", "binarize"],
                default_order: vec!["brightness_contrast", "gamma", "denoise"],
                safety_constraints: vec!["must retain source color channels"],
                preview_cost_level: "low",
                full_quality_cost_level: "low",
            },
            DocumentFilterPreset::Grayscale => FilterPlanContract {
                goal: "convert to neutral Gray8",
                allowed_processors: vec!["grayscale"],
                forbidden_processors: vec!["binarize", "unsharp"],
                default_order: vec!["grayscale"],
                safety_constraints: vec!["use documented luminance coefficients"],
                preview_cost_level: "low",
                full_quality_cost_level: "low",
            },
            DocumentFilterPreset::BlackAndWhite => FilterPlanContract {
                goal: "produce a binary document",
                allowed_processors: vec!["grayscale", "background_normalization", "binarize"],
                forbidden_processors: vec!["unsharp", "neural_restoration"],
                default_order: vec!["grayscale", "background_normalization", "binarize"],
                safety_constraints: vec!["output must contain only zero or 255"],
                preview_cost_level: "low",
                full_quality_cost_level: "medium",
            },
            DocumentFilterPreset::Receipt => placeholder_contract(
                "receipt",
                "receipt-specific low-contrast processing is not implemented in MP4",
            ),
            DocumentFilterPreset::Book => placeholder_contract(
                "book",
                "curved-page and gutter processing is not implemented in MP4",
            ),
            DocumentFilterPreset::Whiteboard => placeholder_contract(
                "whiteboard",
                "whiteboard-specific enhancement is not implemented in MP4",
            ),
            DocumentFilterPreset::PhotoDocument => placeholder_contract(
                "photo_document",
                "semantic photo preservation is not implemented in MP4",
            ),
        }
    }

    /// Applies an explicitly selected baseline filter and preserves all plan diagnostics.
    pub fn apply(
        &self,
        preset: DocumentFilterPreset,
        image: &ImageBuffer,
        conditions: &ConditionVector,
        device_class: DeviceClass,
    ) -> PerceptionResult<RestorationOutput> {
        if preset == DocumentFilterPreset::Original {
            return Ok(RestorationOutput {
                image: image.view().to_owned()?,
                applied_processors: Vec::new(),
                diagnostics: vec!["original_filter_preserved_source_pixels".into()],
            });
        }
        if matches!(
            preset,
            DocumentFilterPreset::Receipt
                | DocumentFilterPreset::Book
                | DocumentFilterPreset::Whiteboard
                | DocumentFilterPreset::PhotoDocument
        ) {
            return Ok(RestorationOutput {
                image: image.view().to_owned()?,
                applied_processors: Vec::new(),
                diagnostics: vec![format!(
                    "{:?}_filter_is_declared_but_not_implemented_in_mp4",
                    preset
                )],
            });
        }
        let decision = self.router.plan(conditions, preset, device_class)?;
        let context = ProcessingContext {
            device_class,
            full_quality: true,
        };
        let mut current = image.view().to_owned()?;
        let mut applied = Vec::new();
        let mut diagnostics = Vec::new();
        for step in decision.plan.steps {
            let Some(processor) = ScalarRestorationProcessor::from_id(&step.processor) else {
                diagnostics.push(format!(
                    "no_scalar_implementation_for={}",
                    step.processor.as_str()
                ));
                continue;
            };
            let output = processor.process(&RestorationInput { image: current }, &context)?;
            current = output.image;
            applied.extend(output.applied_processors);
            diagnostics.extend(output.diagnostics);
        }
        diagnostics.extend(
            decision.plan.skipped.into_iter().map(|skipped| {
                format!("skipped:{}:{}", skipped.processor.as_str(), skipped.reason)
            }),
        );
        Ok(RestorationOutput {
            image: current,
            applied_processors: applied,
            diagnostics,
        })
    }
}

fn placeholder_contract(goal: &'static str, limitation: &'static str) -> FilterPlanContract {
    FilterPlanContract {
        goal,
        allowed_processors: Vec::new(),
        forbidden_processors: vec!["*"],
        default_order: Vec::new(),
        safety_constraints: vec![limitation],
        preview_cost_level: "none",
        full_quality_cost_level: "none",
    }
}
