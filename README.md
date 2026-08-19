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

Current status: MP4 provides the MP2 image/geometry substrate, MP3 quality/detection/fusion/refinement/temporal intelligence, and MP4 canonical planar document reconstruction, surface/dense-warp seams, condition routing, scalar restoration filters, semantic output contracts, and runtime DomainPack registration. Neural restoration, curved-page dewarping, OCR, export, platform bindings, and model integrations remain intentionally deferred.

The repository is Apache-2.0. See [third_party/dependency-register.toml](third_party/dependency-register.toml) and [docs/dependency-policy.md](docs/dependency-policy.md) before adding dependencies.
