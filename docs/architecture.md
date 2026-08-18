# Architecture

The engine follows a domain-neutral staged flow: input, quality analysis, independent detection, fusion, refinement, temporal decision, geometric reconstruction, condition analysis, routing, restoration, semantics, and structured export. Every stage uses explicit input/output contracts from `core`; stages can be skipped only when the selected `DomainPack` declares that choice.

Independent detector outputs retain source, confidence, and uncertainty. Fusion never erases provenance. The router selects processors from condition evidence and can later trade expected benefit against cost and device capability. Python model code stays outside this runtime; production consumes versioned exported artifacts only.

