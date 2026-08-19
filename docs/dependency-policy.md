# Dependency policy

Only permissive, clearly licensed dependencies may be introduced without an explicit architecture review. Before adding one, record its exact version, license, source URL, purpose, and risk in `third_party/dependency-register.toml`; retain notices when required. GPL, AGPL, LGPL, non-commercial, research-only, unclear, and custom licenses are prohibited unless a dedicated license-risk entry and explicit maintainer approval exist. Model weights and datasets require separate provenance audits.

Planned references that lack verified component-level provenance are explicitly `audit-gated` in the register. They are not runtime dependencies and cannot be vendored, linked, or used in a production path until the gate is cleared. This applies to native transitive dependencies, model weights, datasets, and any third-party code port as well as a named upstream repository.
