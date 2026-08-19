# Web binding

The web runtime calls the `wasm-bindgen` API from a worker. It passes decoded pixel buffers, never browser-specific geometry algorithms. The web loader reports `ENGINE_READY` only for a reviewed WASM engine and reports `ENGINE_FAILED` otherwise.

MP10 validates the WASM target build but does not publish a bundled browser package. Therefore loading an absent artifact is a visible production failure, not a fallback to the test-only web mock.
