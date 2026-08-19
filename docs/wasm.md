# WebAssembly binding

`bindings/wasm` is a real `wasm32-unknown-unknown` build target. Build it with:

```powershell
cargo build -p wellfriend-perception-wasm --target wasm32-unknown-unknown
```

The crate uses `wasm-bindgen` to expose `createEngine`, `EngineHandle.analyzeFrame`, `reconstructPage`, `applyFilter`, `destroyEngine`, and `version`. These methods use the same JSON schema as the C ABI. `bindings/wasm/wellfriend_perception_wasm.d.ts` records the generated API shape.

MP10 verifies the Rust WASM target, not a published npm package or browser-device latency. A web product must fail closed if the reviewed artifact cannot load; it must not switch to the dev mock or a TypeScript detector.
