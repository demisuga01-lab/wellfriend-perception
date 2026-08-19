# DomainPack contract

A `DomainPack` declares a stable identifier, the generic pipeline stages it supports, and its structured-output schema. A pack may add domain metrics, candidate kinds, geometry models, processor graphs, semantics, and exporters, but must not change generic types or hide third-party behavior. `document` is the primary reference pack; whiteboard, ID-card, industrial, medical, satellite, and photogrammetry packs are optional and independently versioned.

MP3 detectors plug in through `PerceptionDetector`: they declare capabilities, consume a `DetectorInput`, and return source-attributed candidates with score, bounded confidence, uncertainty, timing, diagnostics, and typed geometry. Fusion retains every contributing and rejected source. The document pack may add page coverage/visibility/shadow metrics but keeps those extensions out of generic quality semantics. Manual geometry enters as `DetectionSource::Manual`, is validated as a quad, and may override automatic fusion under the documented policy.

Model-backed detectors are adapters over a checked artifact identifier and a declared inference runtime. They cannot import arbitrary Python and do not make ONNX Runtime, LiteRT, WASM, or native inference a hard dependency of the shared engine.
