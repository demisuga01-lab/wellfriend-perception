# ADR-0023: DomainPack runtime registration

## Decision

Use `RuntimeDomainPack` plus a registry that records supported inputs, reconstruction family, stages, processors, filters, benchmarks, and limitations.

## Consequences

Document is the meaningful MP4 pack. Other packs compile as explicit stubs, so future domains can register without altering generic core semantics or inheriting document assumptions.
