# ADR-0009: Geometry and homography design

## Status

Accepted — MP2.

## Decision

The core exposes small dependency-free vectors/matrices and geometry primitives. `Transform2D` stores a 3×3 projective transform; homography estimation solves the exact four-point correspondence case with a checked partial-pivot system solve. Warps use inverse mapping and explicit nearest/bilinear sampling with explicit borders. Line fitting has deterministic least-squares and RANSAC baselines.

## Consequences

Later document, geospatial, and reconstruction modules share coordinate and error semantics. Exact four-point estimation is intentionally not a substitute for normalized multi-point DLT, robust homography RANSAC, calibration, or bundle adjustment; those require separately benchmarked follow-up decisions.
