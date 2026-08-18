# Repository layout

The workspace intentionally has two production crates in MP2:

- `core` — domain-neutral contracts, checked image storage/views, diagnostics/errors, math, and geometry.
- `image` — scalar-first color conversion, resize, crop/pad, normalization, histograms, convolution, filtering, gradients, and thresholds built on `core`.

Within `core/src`, `math/` contains scalar/vector/matrix/linear-algebra/statistics/robust helpers and `geometry/` contains primitives, lines, polygons, transforms, homographies, interpolation, warps, fitting, RANSAC, and camera placeholders. `prelude.rs` is the compact supported import surface; module-level exports remain available for callers that prefer explicit paths.

The remaining top-level module folders preserve the MP1 implementation seams. `domains` plugs domain behavior into the core, `bindings` exposes stable adapters, `benchmarks` owns schemas and result documentation, and `third_party` owns dependency provenance. They must not duplicate the generic primitives implemented in `core` and `image`.
