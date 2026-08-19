# PINTO-style curved-page dewarping boundary

MP4 treats `PINTO0309/doc-dewarping` as an algorithmic reference only. Its current upstream source is MIT-licensed, but it is not a Rust dependency, no source is vendored, no model/data artifact is included, and production use remains audit-gated. The reference implementation's Python/OpenCV dependency graph and any code port require a separate component-level license and provenance review.

The future Wellfriend surface path may accept text-line and line-segment evidence inside a detected page region, fit a generalized cylindrical surface, estimate camera pose and focal length, solve bounded nonlinear least squares, reject outliers, and render through an inverse mapping. MP4 provides only the safe seams needed for that work: curvature evidence, `SurfaceGrid`, `MeshWarp`, `DenseWarpField`, mesh-to-field conversion, remapping, and explicit curved-page routing.

No caller may quietly run a curved page through the planar path after curvature is classified as mild or strong. It must either select an audited surface reconstructor or return the `SurfaceUnavailable` diagnostic.
