# Browser WASM package

Build a browser-loadable package, not just a raw `wasm32` object:

```powershell
.\scripts\build-wasm.ps1 -Profile release
.\scripts\validate-runtime-artifact.ps1 -ArtifactRoot target\wellfriend-wasm -Kind wasm
```

The package contains `wellfriend_perception_bg.wasm`, wasm-bindgen JavaScript and
TypeScript declarations, a module marker, manifest, checksums, and README. The
manifest locks the source SHA, Rust/wasm-bindgen versions, target, profile, runtime
schema and checked files. Browser consumers must load a verified local package; they
must not fetch arbitrary WASM URLs or silently use a development mock.

The manual **Browser WASM package** workflow produces an uploadable CI artifact.
This remains a scalar experimental runtime, not a performance or scanner-quality
claim.
