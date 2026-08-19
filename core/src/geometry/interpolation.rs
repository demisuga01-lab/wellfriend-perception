//! Scalar sampling helpers shared by resizing and image warping.

use crate::{PerceptionError, PerceptionResult};

/// Checked storage layout required for interleaved scalar sampling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SamplingLayout {
    /// Logical image width in pixels.
    pub width: u32,
    /// Logical image height in pixels.
    pub height: u32,
    /// Bytes between starts of adjacent rows.
    pub stride: usize,
    /// Interleaved components per pixel.
    pub channels: usize,
}

/// Nearest-neighbor sampling for one interleaved u8 channel plane.
pub fn sample_nearest(
    data: &[u8],
    layout: SamplingLayout,
    x: f32,
    y: f32,
    channel: usize,
) -> Option<u8> {
    if channel >= layout.channels || !x.is_finite() || !y.is_finite() {
        return None;
    }
    let sx = x.round() as i32;
    let sy = y.round() as i32;
    if sx < 0 || sy < 0 || sx >= layout.width as i32 || sy >= layout.height as i32 {
        return None;
    }
    data.get(sy as usize * layout.stride + sx as usize * layout.channels + channel)
        .copied()
}

/// Bilinear sampling with a caller-provided fallback for out-of-bounds neighbors.
pub fn sample_bilinear(
    data: &[u8],
    layout: SamplingLayout,
    x: f32,
    y: f32,
    channel: usize,
    fallback: u8,
) -> PerceptionResult<u8> {
    if channel >= layout.channels {
        return Err(PerceptionError::OutOfBounds {
            reason: "sample channel exceeds image channels".into(),
        });
    }
    if !x.is_finite() || !y.is_finite() {
        return Err(PerceptionError::NumericFailure {
            reason: "sample coordinate is non-finite".into(),
        });
    }
    let x0 = x.floor();
    let y0 = y.floor();
    let dx = x - x0;
    let dy = y - y0;
    let fetch = |sx: i32, sy: i32| -> u8 {
        if sx < 0 || sy < 0 || sx >= layout.width as i32 || sy >= layout.height as i32 {
            fallback
        } else {
            data.get(sy as usize * layout.stride + sx as usize * layout.channels + channel)
                .copied()
                .unwrap_or(fallback)
        }
    };
    let top = f32::from(fetch(x0 as i32, y0 as i32)) * (1.0 - dx)
        + f32::from(fetch(x0 as i32 + 1, y0 as i32)) * dx;
    let bottom = f32::from(fetch(x0 as i32, y0 as i32 + 1)) * (1.0 - dx)
        + f32::from(fetch(x0 as i32 + 1, y0 as i32 + 1)) * dx;
    Ok((top * (1.0 - dy) + bottom * dy).round().clamp(0.0, 255.0) as u8)
}
