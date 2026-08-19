# ADR-0012: Benchmark-first optimization policy

## Status

Accepted — MP2.

## Decision

The repository keeps a deterministic, dependency-free baseline harness and ScanBench envelope before optimizing core primitives. CI build-validates the harness; a manual workflow runs it. Benchmarks are coupled with correctness/equivalence tests, not used as the only acceptance condition.

## Consequences

Performance changes have a repeatable starting point and must disclose fixture, device class, iterations, and implementation path. Hosted-runner outputs are trend signals only; product device latency, memory, and battery claims require dedicated controlled measurements.
