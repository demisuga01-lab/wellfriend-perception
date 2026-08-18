# Architecture

`wellfriend-perception` is a domain-neutral Rust workspace. The engine follows a staged flow: input, quality analysis, independent detection, fusion, refinement, temporal decision, geometric reconstruction, condition analysis, routing, restoration, semantics, and structured export. A selected `DomainPack` supplies domain behavior; generic modules never infer that the input is a page, camera frame, medical volume, or satellite raster.

## MP2 foundation

The `core` crate owns validated data and mathematical contracts. `Observation` can carry image, sequence, sensor, raster, or volume descriptors alongside source/device/processing metadata. `Confidence`, `Score`, `Probability`, and `Reliability` reject non-finite values and values outside `[0, 1]`; `Uncertainty` preserves a variance extension seam. `Diagnostic`, `StageTiming`, and `PipelineTrace` provide lightweight provenance without binding a UI or logging backend.

`ImageBuffer` owns checked interleaved storage; `ImageView` and `ImageViewMut` expose bounded row access and ROI views. Shapes, strides, regions, format layouts, and arithmetic overflow are validated before data access. Scalar image operators use row-aware access so stride padding is never interpreted as pixels.

The `math` module supplies dependency-free vectors, small matrices, homogeneous coordinates, deterministic statistics, a partial-pivot linear solve, and deterministic pseudo-random sampling. The `geometry` module layers points, lines, segments, polygons, quads, transforms, exact four-point homography estimation, interpolation, projective warping, dense remapping, and line fitting/RANSAC on these primitives. These tools are suitable foundations for later planar, surface, geospatial, and photogrammetric modules but do not themselves select a domain algorithm.

## Boundaries and evolution

Independent detector outputs retain source, confidence, and uncertainty. Fusion must not erase provenance. The router can later trade expected benefit against cost and device capability. Python model research remains outside this runtime; production consumes auditable exported artifacts only. MP2 does not integrate models, OpenCV, platform bindings, camera APIs, OCR, or document-specific detection.

Public APIs return `PerceptionResult` for recoverable failures. Library code forbids unsafe Rust and classifies invalid buffers, unsupported formats, out-of-bounds access, numerical degeneracy, non-invertible transforms, insufficient points, and invalid confidence distinctly.
