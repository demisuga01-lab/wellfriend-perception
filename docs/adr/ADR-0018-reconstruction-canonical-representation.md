# ADR-0018: Reconstruction output and canonical representation model

## Decision

Use typed reconstruction contracts and a generic canonical representation that carries family, geometry, material artifacts, confidence, quality, and diagnostics. `CanonicalDocument` is a document-domain wrapper over this boundary.

## Consequences

Detection geometry remains evidence; reconstruction creates the representation consumed by restoration, semantics, and export. Volumetric, geospatial, and photogrammetric families can register without document fields in core.
