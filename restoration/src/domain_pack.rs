//! Runtime DomainPack registration without document assumptions in generic contracts.

use std::collections::BTreeMap;

use wellfriend_perception_core::{PerceptionError, PerceptionResult, PipelineStage};
use wellfriend_perception_reconstruction::{ReconstructionFamily, ReconstructionPlaceholder};

use crate::{DocumentFilterPreset, ProcessorId};

/// Generic payload kinds accepted by a runtime domain pack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportedInputKind {
    /// Standard 2D image.
    Image,
    /// Sensor observation.
    Sensor,
    /// Multi-band or tiled raster.
    Raster,
    /// 3D volume.
    Volume,
}

/// Concrete runtime description of a domain pack.
pub trait RuntimeDomainPack: Send + Sync {
    /// Stable pack id.
    fn id(&self) -> &'static str;
    /// Inputs this pack can represent.
    fn supported_input_kinds(&self) -> &[SupportedInputKind];
    /// Reconstruction family selected by this pack.
    fn reconstruction_family(&self) -> ReconstructionFamily;
    /// Pipeline stages this pack may contribute.
    fn supported_stages(&self) -> &[PipelineStage];
    /// Processor ids owned or allowed by this pack.
    fn processors(&self) -> &[ProcessorId];
    /// Filter presets exposed by this pack.
    fn document_filters(&self) -> &[DocumentFilterPreset];
    /// Domain-owned benchmark metric names.
    fn benchmark_metrics(&self) -> &[&'static str];
    /// Explicit notices for unimplemented components.
    fn diagnostics(&self) -> &[String];
}

/// Registration collection used by a host without hard-coded domain behavior.
#[derive(Default)]
pub struct DomainPackRegistry {
    packs: BTreeMap<String, Box<dyn RuntimeDomainPack>>,
}

impl DomainPackRegistry {
    /// Registers a pack once; duplicate ids return a structured error.
    pub fn register(&mut self, pack: Box<dyn RuntimeDomainPack>) -> PerceptionResult<()> {
        let id = pack.id().to_owned();
        if self.packs.contains_key(&id) {
            return Err(PerceptionError::UnsupportedOperation {
                operation: "duplicate domain pack id",
            });
        }
        self.packs.insert(id, pack);
        Ok(())
    }
    /// Looks up a runtime pack by stable identifier.
    pub fn get(&self, id: &str) -> PerceptionResult<&dyn RuntimeDomainPack> {
        self.packs
            .get(id)
            .map(Box::as_ref)
            .ok_or(PerceptionError::UnsupportedOperation {
                operation: "unknown domain pack",
            })
    }
    /// Registered pack ids in deterministic sort order.
    pub fn ids(&self) -> Vec<&str> {
        self.packs.keys().map(String::as_str).collect()
    }
}

/// The reference domain pack with real MP4 planar/filter registration.
#[derive(Clone, Debug, Default)]
pub struct DocumentDomainPack {
    diagnostics: Vec<String>,
}

impl DocumentDomainPack {
    /// Creates a document pack with its explicit MP4 implementation boundary.
    pub fn new() -> Self {
        Self {
            diagnostics: vec![
                "planar reconstruction and scalar filters are implemented in MP4".into(),
                "curved dewarping, OCR, and export remain separate future stages".into(),
            ],
        }
    }
}

impl RuntimeDomainPack for DocumentDomainPack {
    fn id(&self) -> &'static str {
        "document"
    }
    fn supported_input_kinds(&self) -> &[SupportedInputKind] {
        &[SupportedInputKind::Image]
    }
    fn reconstruction_family(&self) -> ReconstructionFamily {
        ReconstructionFamily::Planar
    }
    fn supported_stages(&self) -> &[PipelineStage] {
        &[
            PipelineStage::Quality,
            PipelineStage::Detection,
            PipelineStage::Fusion,
            PipelineStage::Refinement,
            PipelineStage::Reconstruction,
            PipelineStage::Routing,
            PipelineStage::Restoration,
            PipelineStage::Semantics,
            PipelineStage::Export,
        ]
    }
    fn processors(&self) -> &[ProcessorId] {
        &[]
    }
    fn document_filters(&self) -> &[DocumentFilterPreset] {
        &[
            DocumentFilterPreset::Original,
            DocumentFilterPreset::Auto,
            DocumentFilterPreset::Clean,
            DocumentFilterPreset::Color,
            DocumentFilterPreset::Grayscale,
            DocumentFilterPreset::BlackAndWhite,
            DocumentFilterPreset::Receipt,
            DocumentFilterPreset::Book,
            DocumentFilterPreset::Whiteboard,
            DocumentFilterPreset::PhotoDocument,
        ]
    }
    fn benchmark_metrics(&self) -> &[&'static str] {
        &[
            "reconstruction_latency",
            "aspect_error",
            "coverage_loss",
            "restoration_delta_contrast",
            "binarization_foreground_ratio",
        ]
    }
    fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}

/// Interface-only pack for a future optional domain.
#[derive(Clone, Debug)]
pub struct StubDomainPack {
    id: &'static str,
    inputs: Vec<SupportedInputKind>,
    family: ReconstructionFamily,
    diagnostics: Vec<String>,
}

impl StubDomainPack {
    /// Builds a declared placeholder that does not claim algorithmic support.
    pub fn new(
        id: &'static str,
        inputs: Vec<SupportedInputKind>,
        family: ReconstructionFamily,
    ) -> Self {
        let placeholder = ReconstructionPlaceholder::new(
            family,
            format!("{id} reconstruction is an MP4 interface-only placeholder"),
        );
        Self {
            id,
            inputs,
            family,
            diagnostics: vec![placeholder.diagnostic],
        }
    }
}

impl RuntimeDomainPack for StubDomainPack {
    fn id(&self) -> &'static str {
        self.id
    }
    fn supported_input_kinds(&self) -> &[SupportedInputKind] {
        &self.inputs
    }
    fn reconstruction_family(&self) -> ReconstructionFamily {
        self.family
    }
    fn supported_stages(&self) -> &[PipelineStage] {
        &[PipelineStage::Reconstruction]
    }
    fn processors(&self) -> &[ProcessorId] {
        &[]
    }
    fn document_filters(&self) -> &[DocumentFilterPreset] {
        &[]
    }
    fn benchmark_metrics(&self) -> &[&'static str] {
        &[]
    }
    fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}

/// Whiteboard registration seam.
pub fn whiteboard_domain_pack() -> StubDomainPack {
    StubDomainPack::new(
        "whiteboard",
        vec![SupportedInputKind::Image],
        ReconstructionFamily::Planar,
    )
}
/// ID-card registration seam.
pub fn id_card_domain_pack() -> StubDomainPack {
    StubDomainPack::new(
        "id_card",
        vec![SupportedInputKind::Image],
        ReconstructionFamily::Planar,
    )
}
/// Industrial anomaly-registration seam.
pub fn industrial_domain_pack() -> StubDomainPack {
    StubDomainPack::new(
        "industrial",
        vec![SupportedInputKind::Image, SupportedInputKind::Sensor],
        ReconstructionFamily::Surface,
    )
}
/// Research-only medical volume-registration seam.
pub fn medical_research_domain_pack() -> StubDomainPack {
    StubDomainPack::new(
        "medical_research",
        vec![SupportedInputKind::Volume],
        ReconstructionFamily::Volumetric,
    )
}
/// Satellite raster/geospatial registration seam.
pub fn satellite_domain_pack() -> StubDomainPack {
    StubDomainPack::new(
        "satellite",
        vec![SupportedInputKind::Raster],
        ReconstructionFamily::Geospatial,
    )
}
/// Photogrammetric image/sensor registration seam.
pub fn photogrammetry_domain_pack() -> StubDomainPack {
    StubDomainPack::new(
        "photogrammetry",
        vec![SupportedInputKind::Image, SupportedInputKind::Sensor],
        ReconstructionFamily::Photogrammetric,
    )
}
