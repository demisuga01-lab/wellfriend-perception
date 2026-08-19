# Security and input handling

The core rejects invalid dimensions, strides, buffers, confidence values, degenerate geometry, and non-invertible numeric operations through structured errors. Consumers must enforce transport file-size limits before decoding.

Future native/WASM bindings must preserve these checks, bound allocation sizes, validate artifact hashes before model execution, and treat model/dataset metadata as untrusted input. Report vulnerabilities through the repository security policy; do not include exploit payloads in public issues.
