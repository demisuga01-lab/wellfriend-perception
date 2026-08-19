# DomainPack contract

A `DomainPack` declares a stable identifier, the generic pipeline stages it supports, and its structured-output schema. A pack may add domain metrics, candidate kinds, geometry models, processor graphs, semantics, and exporters, but must not change generic types or hide third-party behavior. `document` is the primary reference pack; whiteboard, ID-card, industrial, medical, satellite, and photogrammetry packs are optional and independently versioned.

MP3 detectors plug in through `PerceptionDetector`: they declare capabilities, consume a `DetectorInput`, and return source-attributed candidates with score, bounded confidence, uncertainty, timing, diagnostics, and typed geometry. Fusion retains every contributing and rejected source. The document pack may add page coverage/visibility/shadow metrics but keeps those extensions out of generic quality semantics. Manual geometry enters as `DetectionSource::Manual`, is validated as a quad, and may override automatic fusion under the documented policy.

Model-backed detectors are adapters over a checked artifact identifier and a declared inference runtime. They cannot import arbitrary Python and do not make ONNX Runtime, LiteRT, WASM, or native inference a hard dependency of the shared engine.

## MP4 runtime registration

`RuntimeDomainPack` makes the MP1 contract executable without making core document-specific. A pack declares its id, supported input kinds, reconstruction family, supported stages, processor ids, document-filter presets, benchmark metrics, and explicit diagnostics. `DomainPackRegistry` rejects duplicate ids and returns a structured unknown-pack error.

`DocumentDomainPack` registers the implemented planar reconstruction and scalar filter boundary. Whiteboard and ID-card register planar seams; industrial registers a surface/anomaly seam; medical research registers a volumetric seam; satellite registers a geospatial seam; and photogrammetry registers a pose/surface seam. These stubs prove registration compatibility only. They do not claim algorithms, clinical suitability, geospatial correctness, or model availability.
