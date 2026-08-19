# Boundary detection policy

Wellfriend’s product target is best-available boundary detection for documents, objects, lines, boxes, rounded objects, and arbitrary shapes. It is not a promise that all images contain recoverable edge information.

When visible evidence is strong, the SDK returns a boundary with confidence, uncertainty, edge support, provenance, and diagnostics. With weak or ambiguous evidence it returns a lower-confidence estimate and may return multiple candidates. With occlusion, saturation, out-of-frame geometry, blur, or low contrast, it returns `insufficient_evidence` and/or `manual_required` rather than fabricating an unseen edge.

Manual geometry is validated evidence and retains `manual` provenance. A caller must not treat heuristic scalar confidence as calibrated probability. Current MP10 detection is document-quad focused; the `BoundaryKind` and `BoundaryGeometry` contracts additionally represent points, lines, segments, polygons, circles, ellipses, freeform contours, masks, and surface outlines for future domain packs.
