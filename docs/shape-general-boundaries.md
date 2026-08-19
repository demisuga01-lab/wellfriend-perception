# Shape-general boundaries

`BoundaryDetectionMode` declares whether a caller needs a document page, object outline, geometric shape, line art, arbitrary edge, or automatic mode. `BoundaryResult` keeps geometry separate from evidence: `BoundaryKind`, optional `BoundaryGeometry`, bounded confidence, extensible uncertainty, edge support, source provenance, status values, and limitations.

This contract does not claim arbitrary-shape segmentation is implemented in MP10. Unsupported future shapes remain explicit rather than being coerced to a quad.
