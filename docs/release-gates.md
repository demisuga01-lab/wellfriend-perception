# v0.1.0-alpha.2 release gates

| Capability | Status | Evidence | Alpha claim |
| --- | --- | --- | --- |
| Rust image/math/geometry core | implemented | workspace tests and benchmarks compile | core foundation |
| Classical document detection | experimental synthetic baseline | MP3 tests/ScanBench fixtures | not device validated |
| Planar reconstruction and scalar filters | experimental synthetic baseline | MP4 tests | not production scan quality |
| DomainPack runtime | implemented contract | domain proof registrations | extensibility proof |
| C ABI scalar runtime | implemented/host-tested | FFI parity tests | experimental scalar bridge |
| WASM scalar runtime | packaged CI/manual artifact | wasm-bindgen package manifest/checksums and smoke path | scalar experimental runtime |
| Android scanner | packaged CI/manual ABI artifact | arm64-v8a/x86_64 manifest/checksums; scanner sync path | native packaging, no device claim |
| Web/desktop scanner | packaged WASM sync path / fail-closed production | local package loader and worker contracts | browser package required |
| Models registry | contract-only/experimental | registry validation | no weights released |
| OCR/PDF/real-device benchmarks | blocked | not implemented or measured | no claim |

Alpha release is permitted only as an unstable technical preview. It is not a production scanner, medical product, OCR system, or benchmark winner.
