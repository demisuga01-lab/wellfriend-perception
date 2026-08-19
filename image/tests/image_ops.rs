use wellfriend_perception_core::{
    ImageBuffer, ImageShape, PerceptionError, PixelFormat, RegionOfInterest, Stride,
};
use wellfriend_perception_image::*;

fn gray(width: u32, height: u32, data: Vec<u8>) -> ImageBuffer {
    ImageBuffer::new(width, height, PixelFormat::Gray8, data).unwrap()
}

#[test]
fn color_conversion_and_normalized_formats_are_correct() {
    let rgb = ImageBuffer::new(1, 1, PixelFormat::Rgb8, vec![255, 0, 0]).unwrap();
    let bgr = convert_color(&rgb, PixelFormat::Bgr8).unwrap();
    assert_eq!(bgr.as_bytes(), &[0, 0, 255]);
    let grayscale = convert_color(&rgb, PixelFormat::Gray8).unwrap();
    assert_eq!(grayscale.as_bytes(), &[54]); // BT.709 red luminance, rounded.
    let restored = convert_color(&grayscale, PixelFormat::Rgb8).unwrap();
    assert_eq!(restored.as_bytes(), &[54, 54, 54]);

    let rgba = ImageBuffer::new(1, 1, PixelFormat::Rgba8, vec![4, 5, 6, 7]).unwrap();
    assert_eq!(
        convert_color(&rgba, PixelFormat::Rgb8).unwrap().as_bytes(),
        &[4, 5, 6]
    );
    let floats = u8_to_f32_normalized(&rgb).unwrap();
    assert_eq!(floats, vec![1.0, 0.0, 0.0]);
}

#[test]
fn crop_padding_resize_and_stride_sensitive_ops_are_correct() {
    let input = gray(3, 2, vec![1, 2, 3, 4, 5, 6]);
    let cropped = crop(
        &input,
        RegionOfInterest::within(input.shape(), 1, 0, 2, 2).unwrap(),
    )
    .unwrap();
    assert_eq!(cropped.as_bytes(), &[2, 3, 5, 6]);
    let padded = pad(&input, 1, 1, 1, 1, BorderMode::Replicate).unwrap();
    assert_eq!(padded.shape().width, 5);
    assert_eq!(
        padded.as_bytes(),
        &[1, 1, 2, 3, 3, 1, 1, 2, 3, 3, 4, 4, 5, 6, 6, 4, 4, 5, 6, 6]
    );
    assert_eq!(resize_nearest(&input, 6, 4).unwrap().shape().width, 6);
    let bilinear = resize_bilinear(&input, 2, 2).unwrap();
    assert_eq!(bilinear.as_bytes(), &[1, 3, 4, 6]);

    let strided = ImageBuffer::new_with_stride(
        ImageShape::new(2, 2).unwrap(),
        PixelFormat::Gray8,
        Stride(4),
        vec![10, 20, 99, 99, 30, 40, 99, 99],
    )
    .unwrap();
    assert_eq!(histogram_gray(&strided).unwrap().bins[99], 0);
    assert_eq!(
        scale_to_unit_gray(&strided).unwrap(),
        vec![10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0, 40.0 / 255.0]
    );
    assert_eq!(
        threshold_gray(&strided, 25).unwrap().as_bytes(),
        &[0, 0, 255, 255]
    );
}

#[test]
fn histogram_convolution_filters_gradients_and_thresholds_are_correct() {
    let input = gray(3, 3, vec![0, 0, 0, 0, 255, 0, 0, 0, 0]);
    let histogram = histogram_gray(&input).unwrap();
    assert_eq!(histogram.bins[0], 8);
    assert_eq!(histogram.bins[255], 1);
    assert_eq!(percentile_from_histogram(&histogram, 1.0).unwrap(), 255);
    assert_eq!(cumulative_histogram(&histogram)[255], 9);

    let convolved = convolve_gray(
        &input,
        &kernels::box3().unwrap(),
        &BorderMode::Constant(vec![0]),
    )
    .unwrap();
    assert!((convolved[4] - 255.0 / 9.0).abs() < 0.01);
    assert_eq!(median_blur_3x3(&input).unwrap().get_u8(1, 1, 0).unwrap(), 0);
    assert!(
        gradient_magnitude(&input)
            .unwrap()
            .iter()
            .any(|value| *value > 0.0)
    );
    assert!(laplacian(&input).unwrap().iter().any(|value| *value != 0.0));

    let bimodal = gray(4, 1, vec![0, 0, 255, 255]);
    let otsu = otsu_threshold(&bimodal).unwrap();
    assert!(otsu < 255);
    assert_eq!(
        threshold_gray(&bimodal, 128).unwrap().as_bytes(),
        &[0, 0, 255, 255]
    );
    assert_eq!(
        adaptive_mean_threshold(&bimodal, 3, 0.0)
            .unwrap()
            .shape()
            .width,
        4
    );
}

#[test]
fn normalization_and_error_paths_are_explicit() {
    let input = gray(3, 1, vec![10, 20, 30]);
    assert_eq!(
        min_max_normalize_gray(&input).unwrap().as_bytes(),
        &[0, 127, 255]
    );
    assert_eq!(
        clamp_gray(&input, 15, 25).unwrap().as_bytes(),
        &[15, 20, 25]
    );
    assert_eq!(
        gamma_gray(&input, 1.0).unwrap().as_bytes(),
        input.as_bytes()
    );
    assert_eq!(
        mean_std_normalize_gray(&input, 20.0, 10.0).unwrap(),
        vec![-1.0, 0.0, 1.0]
    );
    assert!(matches!(
        resize_nearest(&input, 0, 1),
        Err(PerceptionError::InvalidDimensions { .. })
    ));
    let rgb = ImageBuffer::new(1, 1, PixelFormat::Rgb8, vec![1, 2, 3]).unwrap();
    assert!(matches!(
        histogram_gray(&rgb),
        Err(PerceptionError::UnsupportedFormat { .. })
    ));
}
