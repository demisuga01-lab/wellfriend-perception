# ADR-0013: Quality report and guidance model

## Status

Accepted — MP3.

## Decision

Quality analyzers report raw measurements, normalized higher-is-better scores, estimate confidence, warnings, and machine-readable recommended actions. Generic metrics remain image/sensor neutral. Document coverage, visibility, shadow, curvature, and perspective extensions live in the document domain layer.

## Consequences

Platform UI can localize guidance without importing detector internals. Scalar glare, motion, occlusion, curvature, and residual-perspective placeholders are explicitly diagnosed and must not be treated as calibrated classifiers.
