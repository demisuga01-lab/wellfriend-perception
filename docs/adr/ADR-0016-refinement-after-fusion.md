# ADR-0016: Refinement after fusion strategy

## Status

Accepted — MP3.

## Decision

High-resolution edge/corner refinement runs after consensus, not independently inside every detector. The scalar baseline searches edge-normal gradient maxima, robustly fits each edge, and intersects adjacent lines. Weak or inconsistent evidence returns the validated coarse quad with reduced confidence.

## Consequences

Refinement cannot amplify detector disagreement. Fractional line intersections provide a baseline subpixel coordinate estimate; gradient interpolation and multi-scale refinement remain future work.
