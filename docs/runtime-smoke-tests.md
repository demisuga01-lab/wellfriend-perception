# Runtime smoke tests

The runtime smoke suite uses deterministic synthetic fixtures only. It verifies
direct Rust and C ABI schema/guidance parity, then validates the browser package
and Android ABI manifests/checksums. Build a WASM package locally to run its
browser-loader smoke through `wellfriend-scan`.

These tests prove package wiring and scalar behavior only. They do not measure
real-device latency, camera integration, Google ML Kit parity, OCR, PDF output,
or perfect boundary detection.
