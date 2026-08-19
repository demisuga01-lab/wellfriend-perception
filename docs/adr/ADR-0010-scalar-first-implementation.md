# ADR-0010: Scalar-first implementation strategy

## Status

Accepted — MP2.

## Decision

MP2 implements safe scalar reference algorithms before SIMD, GPU, native-library, or platform-specific acceleration. Resize, filters, histograms, warps, and fitting use deterministic operations and inline fixtures for correctness coverage.

## Consequences

The project gains a portable equivalence oracle and avoids prematurely coupling the shared core to a native runtime. Scalar implementations are not performance commitments. Any optimized replacement must preserve format, border, coordinate, diagnostics, and error behavior and add equivalence plus benchmark evidence.
