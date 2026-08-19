# ADR-0022: Scalar restoration before neural restoration

## Decision

Implement checked scalar baseline processors before neural restoration adapters: grayscale, normalization, gamma, denoise, unsharp, background normalization, and thresholding.

## Consequences

Every filter has deterministic fallback behavior and no model/runtime dependency. Neural processors must preserve the same restoration contract and enter only through audited artifacts.
