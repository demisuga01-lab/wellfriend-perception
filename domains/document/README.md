# Document domain

The future document scanner combines CameraX/Camera2 capture, a MakeACopy-style Android foundation, DocAligner-style corner prediction, OpenCV-style geometry, PINTO-style curved-page dewarping, DocRes-style restoration, PaddleOCR-style semantics, and Wellfriend PDF export. These are architecture references only: no third-party code, models, or weights are vendored.

MP3 adds the first executable reference implementation in `intelligence/src/domains/document.rs`: a scalar bright-component/edge/line quad detector and document quality extensions. It is deliberately limited to generated baseline cases; it does not include third-party detector code, model inference, OCR, restoration, platform capture, or export.

MP4 adds `reconstruction/src/document.rs`, which produces a `CanonicalDocument` / `CanonicalPage` from a validated fused/refined quad using shared homography and warp primitives. It supports free, preset, fallback, and manual aspect policies; preserve/long-edge/manual orientation policies; bounds-checked margins; source-to-page traces; and post-warp quality. `restoration` adds scalar Original, Grayscale, B&W, Clean, and Color baseline filters as explicit plans. Curved pages are routed to a visible unsupported surface path rather than silently being represented as flat. OCR, PDF export, DocRes, and any PINTO-derived implementation remain deferred.
