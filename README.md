# Wellfriend Perception

`wellfriend-perception` is the reusable, Rust-first perception and reconstruction engine for the Wellfriend ecosystem. It owns domain-neutral pipeline contracts and safe baseline image operations. It is not an Android app, an OCR product, a model-training repository, or a vendor drop of third-party libraries.

The first reference domain is document scanning, but the core deliberately does not contain page, OCR, camera, or PDF assumptions. Domain packs supply those details through the documented contract.

## Relationship to the ecosystem

- `wellfriend-models` trains, evaluates, validates, and exports auditable model artifacts.
- `wellfriend-scan` is the Kotlin/Compose, web, and desktop reference product that consumes this engine through bindings.

## Build and test

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
cargo bench --workspace --no-run
```

Current status: MP2 provides checked image buffers/views, domain-neutral math and geometry, scalar image operations, homography/warp, deterministic line RANSAC, and baseline benchmarks. Production detector ensembles, platform bindings, model integrations, and optimized kernels are intentionally deferred.

The repository is Apache-2.0. See [third_party/dependency-register.toml](third_party/dependency-register.toml) and [docs/dependency-policy.md](docs/dependency-policy.md) before adding dependencies.
