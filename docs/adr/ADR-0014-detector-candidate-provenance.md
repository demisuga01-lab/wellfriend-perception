# ADR-0014: Detector candidate provenance model

## Status

Accepted — MP3.

## Decision

Every candidate declares source, typed geometry, separate score/confidence, uncertainty, diagnostics, timings, and detector identity. Model-backed detectors are adapter contracts over exported artifacts and declared runtimes; the MP3 mock runtime exists only in tests.

## Consequences

Fusion and UI/debug tools can explain why a candidate was selected. A heuristic confidence is labeled as such and cannot imply model calibration. Production model runtimes can be added without embedding Python or making them a core dependency.
