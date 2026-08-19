# License audit

The workspace is Apache-2.0. Direct MP2 numerical/error/benchmark dependencies are recorded in `third_party/dependency-register.toml` and validated in CI. Planned OpenCV, ONNX Runtime, DocAligner, PINTO-style, DocRes, PaddleOCR, and CameraX references are not vendored. Any unclear code, weight, or dataset remains audit-gated. GPL, AGPL, LGPL, non-commercial, and research-only dependencies are blocked.
