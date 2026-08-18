# Architecture roadmap

MP2 establishes the reusable engine substrate: validated observations and image storage, scalar color/resize/filter primitives, small-matrix math, planar geometry/homographies/warps, deterministic line fitting, and a baseline benchmark harness.

MP3 may build the document detection and capture-readiness layers on these contracts. It must consume generic `ImageBuffer`, `QualityReport`, geometry, diagnostics, and `DomainPack` APIs rather than placing document semantics in `core` or `image`.

Future optimization may add SIMD, platform acceleration, robust DLT homography, adaptive Gaussian thresholding, bicubic/area resize, and model-backed processors only after equivalence/regression measurements demonstrate correctness. The document pack supplies the first end-to-end reference implementation; whiteboard, ID-card, industrial, medical research, satellite/geospatial, and photogrammetry packs must extend it without altering generic core semantics.
