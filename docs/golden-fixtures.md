# Golden fixtures and parity

MP10 runtime tests use deterministic generated grayscale pages rather than external image assets. They exercise a visible page, manually supplied validated quad, and invalid stride. Direct Rust and C ABI paths must agree on schema version, document/no-document decision, candidate count, and capture readiness. The WASM target is build-validated; cross-browser numerical parity remains future work.
