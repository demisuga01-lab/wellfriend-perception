# Repository layout

The workspace intentionally has three production crates after MP3:

- `core` — domain-neutral contracts, checked image storage/views, diagnostics/errors, math, and geometry.
- `image` — scalar-first color conversion, resize, crop/pad, normalization, histograms, convolution, filtering, gradients, and thresholds built on `core`.
- `intelligence` — quality reports, detector contracts, document-reference classical detection, fusion, refinement, temporal tracking, capture readiness, and generated benchmark fixtures built on `core` and `image`.

Within `core/src`, `math/` contains scalar/vector/matrix/linear-algebra/statistics/robust helpers and `geometry/` contains primitives, lines, polygons, transforms, homographies, interpolation, warps, fitting, RANSAC, and camera placeholders. `prelude.rs` is the compact supported import surface; module-level exports remain available for callers that prefer explicit paths.

The remaining top-level module folders preserve the MP1 implementation seams. `domains` plugs domain behavior into the core, `bindings` exposes stable adapters, `benchmarks` owns schemas and result documentation, and `third_party` owns dependency provenance. They must not duplicate the generic primitives implemented in `core` and `image`.

`intelligence/src/domains/document.rs` is the first executable reference-domain layer. The root `domains/document/` directory remains the cross-language architecture and asset seam; it does not contain a second implementation.
