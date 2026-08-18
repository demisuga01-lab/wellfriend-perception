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
    }
}
