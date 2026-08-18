# Benchmark format

Each ScanBench record is JSON or CSV with `schema_version`, sample identifier and hash, domain, runtime/device class, dependency/model versions, and explicit metric units. The envelope schema is `benchmarks/schema/scanbench-record.schema.json`. Benchmarks retain dataset provenance and never redistribute restricted samples.

## MP2 scalar baseline

Run the baseline harness with:

```powershell
cargo bench -p wellfriend-perception-image --bench baseline
```

It emits one JSON line for RGB-to-gray, bilinear resize, crop, 3x3 convolution, Sobel magnitude, histogram, homography application, perspective warp, least-squares line fit, and RANSAC line fit. Every record uses a deterministic inline synthetic fixture and `domain: "generic"`; the elapsed aggregate is useful only as a local regression reference. It is not a cross-device comparison, production latency target, power measurement, or model benchmark.

Normal CI compiles the harness with `cargo bench --workspace --no-run`. The manually dispatched benchmark workflow executes it on a hosted runner. Results must state the Rust/toolchain version, host class, optimization profile, input dimensions, iteration count, and whether scalar, SIMD, or accelerator paths were enabled.

ScanBench for the later document reference pack records quality scores, page coverage/visibility, detector candidate error, fusion diagnostics, corner and reconstruction error, restoration scores, semantic OCR metrics, latency, memory, and battery impact.
