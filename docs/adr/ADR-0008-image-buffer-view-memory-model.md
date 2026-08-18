# ADR-0008: ImageBuffer and ImageView memory model

## Status

Accepted — MP2.

## Decision

`ImageBuffer` owns interleaved pixel bytes and validates its `ImageShape`, `PixelFormat`, `Stride`, and buffer length at construction. `ImageView` and `ImageViewMut` are non-owning bounded views with checked rows; read-only views can produce validated ROI views and packed owned copies. The initial model supports the compact u8/f32 formats needed by MP2 and reserves variants for YUV, U16, F16, and multi-band data.

## Consequences

Operations cannot accidentally treat padding bytes as pixels, and callers get structured errors rather than undefined behavior or unchecked indexing. The design is scalar-first and does not promise zero-copy conversion between all formats. Mutable ROI slicing and hardware-backed buffers remain follow-up work when a concrete binding needs them.
