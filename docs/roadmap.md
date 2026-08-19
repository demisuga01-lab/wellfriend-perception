# Architecture roadmap

MP2 establishes the reusable engine substrate: validated observations and image storage, scalar color/resize/filter primitives, small-matrix math, planar geometry/homographies/warps, deterministic line fitting, and a baseline benchmark harness.

MP3 now provides scalar quality, a bright-component document-quad detector, model-adapter contracts, quad fusion, post-fusion refinement, geometry-only temporal smoothing, and capture-readiness policy. It consumes generic `ImageBuffer`, `QualityReport`, geometry, diagnostics, and `DomainPack` APIs rather than placing document semantics in `core` or `image`.

MP4 now supplies the canonical reconstruction boundary: planar document pages, page policies, reconstruction quality, a lens-correction seam, surface/dense-warp contracts, conditions, deterministic routing, scalar restoration, filter plans, semantic/export shapes, and runtime domain registration. It should add real model inference only through the model artifact adapter, preserve candidate provenance, and verify classical/model equivalence on governed benchmarks.

MP5 may connect audited model artifacts to the existing adapters and restoration processor contracts. It must not replace scalar behavior without controlled equivalence, provenance, device-cost, and license evidence. Curved dewarping remains behind the documented PINTO-style algorithmic boundary until a separately audited implementation and benchmarks exist.

Future optimization may add SIMD, platform acceleration, robust DLT homography, adaptive Gaussian thresholding, bicubic/area resize, and model-backed processors only after equivalence/regression measurements demonstrate correctness. The document pack supplies the first end-to-end reference implementation; whiteboard, ID-card, industrial, medical research, satellite/geospatial, and photogrammetry packs must extend it without altering generic core semantics.
