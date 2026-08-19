# v0.1.0-alpha.2 release gates

| Capability | Status | Evidence | Alpha claim |
| --- | --- | --- | --- |
| Rust image/math/geometry core | implemented | workspace tests and benchmarks compile | core foundation |
| Classical document detection | experimental synthetic baseline | MP3 tests/ScanBench fixtures | not device validated |
| Planar reconstruction and scalar filters | experimental synthetic baseline | MP4 tests | not production scan quality |
| DomainPack runtime | implemented contract | domain proof registrations | extensibility proof |
| C ABI scalar runtime | implemented/host-tested | FFI parity tests | experimental scalar bridge |
| WASM scalar runtime | target-built | `wasm32-unknown-unknown` build | no package published |
| Android scanner | JNI seam / fail-closed release | host contracts; `.so` packaging deferred | no Android native artifact |
| Web/desktop scanner | WASM seam / fail-closed production | TypeScript runtime mapping tests | no browser WASM package |
| Models registry | contract-only/experimental | registry validation | no weights released |
| OCR/PDF/Android ABI package/browser WASM package | blocked | no released distribution | no claim |

Alpha release is permitted only as an unstable technical preview. It is not a production scanner, medical product, OCR system, or benchmark winner.
