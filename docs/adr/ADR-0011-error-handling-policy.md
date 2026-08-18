# ADR-0011: Error handling policy

## Status

Accepted — MP2.

## Decision

Public fallible library operations return `PerceptionResult<T>` with a `PerceptionError` category. The categories distinguish malformed dimensions/buffers/strides, unsupported format/operation, out-of-bounds access, overflow, numeric failure, degeneracy, non-invertibility, insufficient points, and invalid confidence. Unsafe Rust is forbidden by workspace lint.

## Consequences

Callers can surface actionable diagnostics without parsing strings. Internal invariant assertions are limited to values already guaranteed by a checked format branch; malformed external input must return an error rather than panic. A richer telemetry backend may map these categories into `DiagnosticCode` later.
