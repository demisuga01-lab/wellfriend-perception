# C ABI

`bindings/ffi` produces `wellfriend_perception_ffi` and includes [`wellfriend_perception.h`](../bindings/ffi/include/wellfriend_perception.h). It exports `wf_engine_create`, `wf_engine_destroy`, `wf_analyze_frame`, `wf_reconstruct_page`, `wf_apply_filter`, `wf_string_free`, `wf_last_error`, and `wf_version`.

Inputs must be valid pointers for the declared operation; text inputs are NUL-terminated UTF-8. The bridge checks null pointers, JSON size, dimensions, pixel format, stride, checked buffer length, and a 64 MiB decoded image cap before constructing an `ImageBuffer`. Rust panics are contained at the boundary. Every returned `char *` must be released exactly once with `wf_string_free`; `wf_last_error` is borrowed from the engine and must not be freed.

Supported runtime input formats are `Gray8`, `Rgb8`, `Bgr8`, and `Rgba8`. Multi-plane YUV must be converted by the host first. Errors are schema-versioned JSON and never contain image bytes.
