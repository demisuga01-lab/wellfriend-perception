# Runtime bindings

MP10 exposes the same scalar runtime through C and WebAssembly. Both use schema version `1`, raw decoded pixels, and JSON request/response envelopes. Coordinates are source-image pixels with a top-left origin, rightward x-axis, downward y-axis, and page quads ordered TL/TR/BR/BL.

The binding is deliberately a transport boundary, not a duplicate detector. `analyze` delegates to quality analysis, the classical document detector, quad fusion, refinement, temporal evidence, and capture readiness. `reconstruct` delegates to the planar reconstructor. `applyFilter` delegates to scalar document filter plans.

Returned image byte arrays are appropriate for the first correctness-oriented bridge and golden tests, not a final zero-copy mobile/browser transport. Later bindings may optimize transport without changing the schema semantics.

See [c-abi.md](c-abi.md) and [wasm.md](wasm.md). A runtime must return structured errors rather than panic or silently select a dev mock.
