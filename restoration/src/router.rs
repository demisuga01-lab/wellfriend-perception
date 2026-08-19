//! Deterministic condition-to-processor planning with explicit skip reasons.

use std::collections::{BTreeMap, BTreeSet};

use wellfriend_perception_core::{Confidence, PerceptionResult, Score};

use crate::{ConditionKind, ConditionVector, DocumentFilterPreset};

/// Stable identifier for a scalar or future model-backed processor.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessorId(String);

impl ProcessorId {
    /// Creates an identifier from a non-empty stable token.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    /// Stable identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Function a processor may perform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessorCapability {
    /// Adjust global brightness/contrast.
    BrightnessContrast,
    /// Apply a gamma curve.
    Gamma,
    /// Reduce scalar noise.
    Denoise,
    /// Enhance high-frequency detail.
    Sharpen,
    /// Normalize slow background illumination.
    BackgroundNormalization,
    /// Convert color to gray.
    Grayscale,
    /// Apply a binary threshold.
    Binarization,
    /// Future neural glare processor.
    GlareHandling,
    /// Future neural shadow processor.
    ShadowRemoval,
    /// Future neural deblurring processor.
    Deblur,
    /// Future OCR-aware restoration processor.
    OcrAwareRestoration,
}

/// Predictable scalar cost estimate used by routing, not a runtime promise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessorCost {
    /// Relative latency units.
    pub latency_units: u16,
    /// Relative memory units.
    pub memory_units: u16,
}

/// Expected benefit estimate for a condition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProcessorBenefit {
    /// Bounded heuristic benefit.
    pub score: Score,
    /// Bounded reliability of that estimate.
    pub confidence: Confidence,
}

/// Runtime profile supplied by an application/platform adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeviceClass {
    /// Constrained device that should use only inexpensive scalar plans.
    Low,
    /// Typical mobile or desktop CPU profile.
    Mid,
    /// Higher-cost local compute profile.
    High,
    /// Server-side runtime.
    Server,
    /// Browser/WASM profile.
    Web,
    /// No reliable device information was supplied.
    Unknown,
}

/// A processor selection constraint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessorConstraint {
    /// May not run with this processor in a single safe plan.
    ConflictsWith(ProcessorId),
    /// Requires this device class or stronger class.
    MinimumDeviceClass(DeviceClass),
    /// Must not run for a specific document preset.
    ForbiddenForPreset(DocumentFilterPreset),
}

/// Executable processing instruction with selection provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessingStep {
    /// Selected processor.
    pub processor: ProcessorId,
    /// Conditions that motivated the selection.
    pub conditions: Vec<ConditionKind>,
    /// Deterministic explanatory reason.
    pub reason: String,
    /// Predicted scalar cost.
    pub cost: ProcessorCost,
    /// Predicted benefit.
    pub benefit: ProcessorBenefit,
}

/// Why a known processor was not selected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkippedProcessor {
    /// Processor that was considered.
    pub processor: ProcessorId,
    /// Machine-readable skip identifier.
    pub reason: String,
}

/// Ordered, explainable processor plan.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessingPlan {
    /// Selected processors in safe execution order.
    pub steps: Vec<ProcessingStep>,
    /// Processors omitted for transparent reasons.
    pub skipped: Vec<SkippedProcessor>,
}

/// Directed processing order, preserved even for scalar linear plans.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProcessingGraph {
    /// Node identifiers in topological order.
    pub nodes: Vec<ProcessorId>,
    /// Ordered edges as `(before, after)` identifiers.
    pub edges: Vec<(ProcessorId, ProcessorId)>,
}

/// Complete routing result for diagnostics and future UI explanations.
#[derive(Clone, Debug, PartialEq)]
pub struct RoutingDecision {
    /// Chosen device profile.
    pub device_class: DeviceClass,
    /// Selected plan.
    pub plan: ProcessingPlan,
    /// Ordered graph equivalent of the plan.
    pub graph: ProcessingGraph,
}

/// Detailed routing reasoning kept separate from user-facing text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoutingDiagnostics {
    /// Stable decision messages.
    pub messages: Vec<String>,
}

/// Static metadata used to make routing predictable and auditable.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcessorDescriptor {
    /// Stable processor id.
    pub id: ProcessorId,
    /// Capabilities exposed by the processor.
    pub capabilities: Vec<ProcessorCapability>,
    /// Relative scalar cost.
    pub cost: ProcessorCost,
    /// Static constraints.
    pub constraints: Vec<ProcessorConstraint>,
}

/// Deterministic specialist router for scalar MP4 processing plans.
#[derive(Clone, Debug)]
pub struct SpecialistRouter {
    /// Known processor metadata.
    pub processors: BTreeMap<ProcessorId, ProcessorDescriptor>,
}

impl Default for SpecialistRouter {
    fn default() -> Self {
        let descriptors = [
            descriptor(
                "brightness_contrast",
                vec![ProcessorCapability::BrightnessContrast],
                1,
            ),
            descriptor("gamma", vec![ProcessorCapability::Gamma], 1),
            descriptor("denoise", vec![ProcessorCapability::Denoise], 2),
            descriptor("unsharp", vec![ProcessorCapability::Sharpen], 2),
            descriptor(
                "background_normalization",
                vec![ProcessorCapability::BackgroundNormalization],
                2,
            ),
            descriptor("grayscale", vec![ProcessorCapability::Grayscale], 1),
            descriptor("binarize", vec![ProcessorCapability::Binarization], 2),
        ];
        Self {
            processors: descriptors
                .into_iter()
                .map(|descriptor| (descriptor.id.clone(), descriptor))
                .collect(),
        }
    }
}

impl SpecialistRouter {
    /// Selects a safe, ordered plan.  Selection is deterministic for identical inputs.
    pub fn plan(
        &self,
        conditions: &ConditionVector,
        preset: DocumentFilterPreset,
        device_class: DeviceClass,
    ) -> PerceptionResult<RoutingDecision> {
        let mut requested = Vec::new();
        for (kind, evidence) in &conditions.entries {
            if evidence.score.value() >= 0.20 {
                for processor in &evidence.recommended_processors {
                    requested.push((processor.clone(), kind.clone(), evidence.score.value()));
                }
            }
        }
        match preset {
            DocumentFilterPreset::Grayscale => {
                requested.push((
                    ProcessorId::new("grayscale"),
                    ConditionKind::LowContrast,
                    1.0,
                ));
            }
            DocumentFilterPreset::BlackAndWhite => {
                requested.push((
                    ProcessorId::new("grayscale"),
                    ConditionKind::LowContrast,
                    1.0,
                ));
                requested.push((
                    ProcessorId::new("binarize"),
                    ConditionKind::LowContrast,
                    1.0,
                ));
            }
            DocumentFilterPreset::Clean => {
                requested.push((
                    ProcessorId::new("grayscale"),
                    ConditionKind::LowContrast,
                    0.8,
                ));
                requested.push((
                    ProcessorId::new("background_normalization"),
                    ConditionKind::LowContrast,
                    0.8,
                ));
            }
            _ => {}
        }
        let allowed = allowed_processors(preset);
        let mut plan = ProcessingPlan::default();
        let mut selected = BTreeSet::new();
        for id in ordered_processor_ids() {
            let relevant: Vec<_> = requested
                .iter()
                .filter(|(requested_id, _, _)| requested_id == &id)
                .collect();
            if relevant.is_empty() {
                continue;
            }
            if !allowed.contains(id.as_str()) {
                plan.skipped.push(SkippedProcessor {
                    processor: id.clone(),
                    reason: "filter_preset_forbids_processor".into(),
                });
                continue;
            }
            let descriptor = self.processors.get(&id).ok_or(
                wellfriend_perception_core::PerceptionError::UnsupportedOperation {
                    operation: "processor requested without registered descriptor",
                },
            )?;
            if is_expensive_for(device_class, descriptor.cost) {
                plan.skipped.push(SkippedProcessor {
                    processor: id.clone(),
                    reason: "device_class_cost_budget".into(),
                });
                continue;
            }
            if conflicts(&id, &selected) {
                plan.skipped.push(SkippedProcessor {
                    processor: id.clone(),
                    reason: "conflicts_with_selected_processor".into(),
                });
                continue;
            }
            let benefit = relevant.iter().map(|entry| entry.2).fold(0.0_f32, f32::max);
            plan.steps.push(ProcessingStep {
                processor: id.clone(),
                conditions: relevant
                    .iter()
                    .map(|(_, kind, _)| (*kind).clone())
                    .collect(),
                reason: format!("selected_for_{}_conditions", relevant.len()),
                cost: descriptor.cost,
                benefit: ProcessorBenefit {
                    score: Score::new(benefit.clamp(0.0, 1.0))?,
                    confidence: Confidence::new(0.55)?,
                },
            });
            selected.insert(id);
        }
        let graph = ProcessingGraph {
            nodes: plan
                .steps
                .iter()
                .map(|step| step.processor.clone())
                .collect(),
            edges: plan
                .steps
                .windows(2)
                .map(|window| (window[0].processor.clone(), window[1].processor.clone()))
                .collect(),
        };
        Ok(RoutingDecision {
            device_class,
            plan,
            graph,
        })
    }
}

fn descriptor(
    id: &str,
    capabilities: Vec<ProcessorCapability>,
    latency_units: u16,
) -> ProcessorDescriptor {
    ProcessorDescriptor {
        id: ProcessorId::new(id),
        capabilities,
        cost: ProcessorCost {
            latency_units,
            memory_units: latency_units,
        },
        constraints: Vec::new(),
    }
}

fn ordered_processor_ids() -> Vec<ProcessorId> {
    [
        "grayscale",
        "brightness_contrast",
        "gamma",
        "denoise",
        "background_normalization",
        "unsharp",
        "binarize",
    ]
    .into_iter()
    .map(ProcessorId::new)
    .collect()
}

fn allowed_processors(preset: DocumentFilterPreset) -> BTreeSet<&'static str> {
    match preset {
        DocumentFilterPreset::Original => BTreeSet::new(),
        DocumentFilterPreset::Color => ["brightness_contrast", "gamma", "denoise"]
            .into_iter()
            .collect(),
        DocumentFilterPreset::Grayscale => ["grayscale"].into_iter().collect(),
        DocumentFilterPreset::BlackAndWhite => {
            ["grayscale", "background_normalization", "binarize"]
                .into_iter()
                .collect()
        }
        DocumentFilterPreset::Clean => [
            "grayscale",
            "denoise",
            "background_normalization",
            "unsharp",
        ]
        .into_iter()
        .collect(),
        DocumentFilterPreset::Auto => [
            "grayscale",
            "brightness_contrast",
            "gamma",
            "denoise",
            "background_normalization",
            "unsharp",
            "binarize",
        ]
        .into_iter()
        .collect(),
        DocumentFilterPreset::Receipt
        | DocumentFilterPreset::Book
        | DocumentFilterPreset::Whiteboard
        | DocumentFilterPreset::PhotoDocument => BTreeSet::new(),
    }
}

fn is_expensive_for(device: DeviceClass, cost: ProcessorCost) -> bool {
    matches!(device, DeviceClass::Low) && cost.latency_units > 1
}

fn conflicts(id: &ProcessorId, selected: &BTreeSet<ProcessorId>) -> bool {
    (id.as_str() == "binarize" && selected.contains(&ProcessorId::new("unsharp")))
        || (id.as_str() == "unsharp" && selected.contains(&ProcessorId::new("binarize")))
}
