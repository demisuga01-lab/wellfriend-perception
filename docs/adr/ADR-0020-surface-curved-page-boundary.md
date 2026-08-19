# ADR-0020: Surface reconstruction and curved-page boundary

## Decision

Provide validated surface grids, mesh/dense inverse warp seams, and explicit curved-page routing before implementing dewarping. Flat pages use planar reconstruction; detected curvature must surface an unsupported diagnostic until an audited surface implementation exists.

## Consequences

The system does not overclaim curved-page quality. Future PINTO-style work has a stable input/output boundary without vendoring third-party code.
