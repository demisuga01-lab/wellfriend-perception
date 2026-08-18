//! Baseline image operations. These favor correctness and explicit formats over speed.
use wellfriend_perception_core::{ImageBuffer, PixelFormat};

pub fn grayscale(input: &ImageBuffer) -> Result<ImageBuffer, String> {
    match input.pixel_format {
        PixelFormat::Gray8 => Ok(input.clone()),
        PixelFormat::Rgb8 | PixelFormat::Rgba8 => {
            let channels = input.pixel_format.channels();
            let pixels = input
                .data
                .chunks_exact(channels)
                .map(|p| {
                    ((u16::from(p[0]) * 77 + u16::from(p[1]) * 150 + u16::from(p[2]) * 29) / 256)
                        as u8
                })
                .collect();
            ImageBuffer::new(input.width, input.height, PixelFormat::Gray8, pixels)
        }
    }
}

pub fn crop(
    input: &ImageBuffer,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<ImageBuffer, String> {
    if x.checked_add(width).is_none_or(|v| v > input.width)
        || y.checked_add(height).is_none_or(|v| v > input.height)
    {
        return Err("crop exceeds image bounds".into());
    }
    let c = input.pixel_format.channels();
    let mut data = Vec::with_capacity(width as usize * height as usize * c);
    for row in y..y + height {
        let start = (row as usize * input.width as usize + x as usize) * c;
        data.extend_from_slice(&input.data[start..start + width as usize * c]);
    }
    ImageBuffer::new(width, height, input.pixel_format, data)
}

/// Adds a constant-value border around an image without changing its pixel layout.
pub fn pad_constant(
    input: &ImageBuffer,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    value: u8,
) -> Result<ImageBuffer, String> {
    let width = input
        .width
        .checked_add(left)
        .and_then(|v| v.checked_add(right))
        .ok_or("padded width overflow")?;
    let height = input
        .height
        .checked_add(top)
        .and_then(|v| v.checked_add(bottom))
        .ok_or("padded height overflow")?;
    let channels = input.pixel_format.channels();
    let mut data = vec![value; width as usize * height as usize * channels];
    for source_y in 0..input.height {
        let source_start = source_y as usize * input.width as usize * channels;
        let target_start = ((source_y + top) as usize * width as usize + left as usize) * channels;
        data[target_start..target_start + input.width as usize * channels].copy_from_slice(
            &input.data[source_start..source_start + input.width as usize * channels],
        );
    }
    ImageBuffer::new(width, height, input.pixel_format, data)
}

pub fn normalize_gray(input: &ImageBuffer) -> Result<ImageBuffer, String> {
    if input.pixel_format != PixelFormat::Gray8 {
        return Err("normalization currently requires Gray8".into());
    }
    let Some((&min, &max)) = input.data.iter().min().zip(input.data.iter().max()) else {
        return Ok(input.clone());
    };
    if min == max {
        return Ok(input.clone());
    }
    let data = input
        .data
        .iter()
        .map(|v| ((u16::from(*v - min) * 255) / u16::from(max - min)) as u8)
        .collect();
    ImageBuffer::new(input.width, input.height, input.pixel_format, data)
}

pub fn histogram_gray(input: &ImageBuffer) -> Result<[u32; 256], String> {
    if input.pixel_format != PixelFormat::Gray8 {
        return Err("histogram currently requires Gray8".into());
    }
    let mut histogram = [0; 256];
    for value in &input.data {
        histogram[*value as usize] += 1;
    }
    Ok(histogram)
}

pub fn threshold_gray(input: &ImageBuffer, threshold: u8) -> Result<ImageBuffer, String> {
    if input.pixel_format != PixelFormat::Gray8 {
        return Err("threshold currently requires Gray8".into());
    }
    ImageBuffer::new(
        input.width,
        input.height,
        PixelFormat::Gray8,
        input
            .data
            .iter()
            .map(|v| if *v >= threshold { 255 } else { 0 })
            .collect(),
    )
}

/// Applies a zero-padded 3x3 convolution to a Gray8 image and returns signed samples.
pub fn convolve_gray_3x3(input: &ImageBuffer, kernel: [f32; 9]) -> Result<Vec<f32>, String> {
    if input.pixel_format != PixelFormat::Gray8 {
        return Err("convolution currently requires Gray8".into());
    }
    let mut output = vec![0.0; input.width as usize * input.height as usize];
    for y in 0..input.height as i32 {
        for x in 0..input.width as i32 {
            let mut total = 0.0;
            for ky in -1..=1 {
                for kx in -1..=1 {
                    let sx = x + kx;
                    let sy = y + ky;
                    if sx >= 0 && sy >= 0 && sx < input.width as i32 && sy < input.height as i32 {
                        let source = input.data[sy as usize * input.width as usize + sx as usize];
                        total +=
                            f32::from(source) * kernel[(ky + 1) as usize * 3 + (kx + 1) as usize];
                    }
                }
            }
            output[y as usize * input.width as usize + x as usize] = total;
        }
    }
    Ok(output)
}

/// Returns baseline Sobel gradient magnitudes for a Gray8 image.
pub fn sobel_magnitude_gray(input: &ImageBuffer) -> Result<Vec<f32>, String> {
    let x = convolve_gray_3x3(input, [-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0])?;
    let y = convolve_gray_3x3(input, [-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0])?;
    Ok(x.into_iter().zip(y).map(|(gx, gy)| gx.hypot(gy)).collect())
}

/// Baseline nearest-neighbor resize. Convolution and gradients remain pluggable CPU kernels.
pub fn resize_nearest(input: &ImageBuffer, width: u32, height: u32) -> Result<ImageBuffer, String> {
    if width == 0 || height == 0 {
        return Err("output dimensions must be nonzero".into());
    }
    let c = input.pixel_format.channels();
    let mut data = Vec::with_capacity(width as usize * height as usize * c);
    for oy in 0..height {
        for ox in 0..width {
            let sx = ox * input.width / width;
            let sy = oy * input.height / height;
            let start = (sy as usize * input.width as usize + sx as usize) * c;
            data.extend_from_slice(&input.data[start..start + c]);
        }
    }
    ImageBuffer::new(width, height, input.pixel_format, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operations_are_baseline_correct() {
        let image = ImageBuffer::new(2, 2, PixelFormat::Gray8, vec![0, 128, 255, 64]).unwrap();
        assert_eq!(histogram_gray(&image).unwrap()[128], 1);
        assert_eq!(
            threshold_gray(&image, 100).unwrap().data,
            vec![0, 255, 255, 0]
        );
        assert_eq!(resize_nearest(&image, 1, 1).unwrap().data, vec![0]);
        assert_eq!(pad_constant(&image, 1, 1, 1, 1, 7).unwrap().width, 4);
        assert!(
            sobel_magnitude_gray(&image)
                .unwrap()
                .iter()
                .any(|value| *value > 0.0)
        );
    }
}
