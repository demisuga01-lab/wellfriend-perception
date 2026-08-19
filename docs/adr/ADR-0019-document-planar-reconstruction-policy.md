# ADR-0019: Document planar reconstruction policy

## Decision

Use validated fused/refined quads, exact four-point homography, scalar inverse warp, explicit aspect/orientation/margin policies, and post-warp quality reporting for the MP4 reference page path.

## Consequences

The output is deterministic and traceable. Presets are caller-selected policies, not automatic paper classification; physical size is unknown unless a preset or manual input declares it.
