# Wellfriend Perception

`wellfriend-perception` is the reusable, Rust-first perception and reconstruction engine for the Wellfriend ecosystem. It owns domain-neutral pipeline contracts and safe baseline image operations. It is not an Android app, an OCR product, a model-training repository, or a vendor drop of third-party libraries.

The first reference domain is document scanning, but the core deliberately does not contain page, OCR, camera, or PDF assumptions. Domain packs supply those details through the documented contract.

## Relationship to the ecosystem

- `wellfriend-models` trains, evaluates, validates, and exports auditable model artifacts.
- `wellfriend-scan` is the Kotlin/Compose, web, and desktop reference product that consumes this engine through bindings.

## Build and test

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

Current status: MP1 architecture contracts and baseline image primitives are implemented. Production detector ensembles, platform bindings, and optimized kernels are intentionally deferred.

The repository is Apache-2.0. See [third_party/dependency-register.toml](third_party/dependency-register.toml) and [docs/dependency-policy.md](docs/dependency-policy.md) before adding dependencies.

