//! Deterministic scalar baseline measurements for MP2.
//!
//! Run with `cargo bench -p wellfriend-perception-image --bench baseline`.

use std::{hint::black_box, time::Instant};
use wellfriend_perception_core::{
    ImageBuffer, PixelFormat, Point2, RegionOfInterest, Transform2D,
    benchmarks::BenchmarkRecord,
    geometry::{
        RansacConfig, WarpBorder, least_squares_line_fit, ransac_line_fit, warp_perspective,
    },
    math::Vec2,
};
use wellfriend_perception_image::{
    BorderMode, convolve_gray, crop, gradient_magnitude, grayscale, histogram_gray, kernels,
    resize_bilinear,
};

const ITERATIONS: u64 = 100;

fn main() {
    let rgb = ImageBuffer::new(
        64,
        48,
        PixelFormat::Rgb8,
        (0..(64 * 48 * 3))
            .map(|index| (index % 251) as u8)
            .collect(),
    )
    .expect("inline RGB fixture is valid");
    let gray = grayscale(&rgb).expect("baseline grayscale conversion is valid");
    let identity = Transform2D::identity();
    let points: Vec<_> = (0..40)
        .map(|index| Point2::new(index as f32, index as f32 * 1.5 + 2.0))
        .collect();
    let mut outliers = points.clone();
    outliers.extend([Point2::new(4.0, 99.0), Point2::new(30.0, -50.0)]);

    measure("rgb_to_grayscale", || {
        let _ = black_box(grayscale(&rgb).expect("valid"));
    });
    measure("resize_bilinear", || {
        let _ = black_box(resize_bilinear(&gray, 96, 72).expect("valid"));
    });
    measure("crop", || {
        let _ = black_box(
            crop(
                &gray,
                RegionOfInterest::within(gray.shape(), 8, 8, 32, 24).expect("valid"),
            )
            .expect("valid"),
        );
    });
    measure("convolution_3x3", || {
        let _ = black_box(
            convolve_gray(
                &gray,
                &kernels::box3().expect("valid"),
                &BorderMode::Replicate,
            )
            .expect("valid"),
        );
    });
    measure("sobel", || {
        let _ = black_box(gradient_magnitude(&gray).expect("valid"));
    });
    measure("histogram", || {
        let _ = black_box(histogram_gray(&gray).expect("valid"));
    });
    measure("homography_apply", || {
        let _ = black_box(
            identity
                .apply_point(Point2::new(13.0, 21.0))
                .expect("valid"),
        );
    });
    measure("warp_perspective", || {
        let _ = black_box(
            warp_perspective(
                &gray,
                identity,
                gray.shape(),
                wellfriend_perception_core::geometry::SamplingMode::Bilinear,
                WarpBorder::Replicate,
            )
            .expect("valid"),
        );
    });
    measure("least_squares_line_fit", || {
        let _ = black_box(least_squares_line_fit(&points).expect("valid"));
    });
    measure("ransac_line_fit", || {
        let _ = black_box(
            ransac_line_fit(
                &outliers,
                RansacConfig {
                    inlier_threshold: 0.5,
                    iterations: 64,
                    ..RansacConfig::default()
                },
            )
            .expect("valid"),
        );
    });
    black_box(Vec2::new(1.0, 2.0));
}

fn measure(operation: &str, mut action: impl FnMut()) {
    let started = Instant::now();
    for _ in 0..ITERATIONS {
        action();
    }
    let record =
        BenchmarkRecord::scalar_baseline(operation, ITERATIONS, started.elapsed().as_nanos());
    println!("{}", record.to_json_line());
}
