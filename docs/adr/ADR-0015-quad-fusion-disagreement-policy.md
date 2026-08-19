# ADR-0015: Quad fusion and disagreement policy

## Status

Accepted — MP3.

## Decision

Quad consensus uses corner distance, convex overlap, edge orientation, area, and center agreement. Contributing and rejected sources remain visible in `FusionResult`. A validated manual source can explicitly override automatic candidates.

## Consequences

The system avoids opaque averaging across incompatible geometry. Manual correction is authoritative but never bypasses geometric validation. MP3 uses deterministic heuristic weighting; calibrated source reliability is a future benchmarked upgrade.
