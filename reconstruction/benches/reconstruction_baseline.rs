//! Manual scalar reconstruction benchmark; CI only validates that it compiles.

use std::time::Instant;

use wellfriend_perception_core::{
    ImageBuffer, PixelFormat, Point2, Quad, benchmarks::BenchmarkRecord,
};
use wellfriend_perception_reconstruction::{
    CropMarginPolicy, MeshWarp, OrientationPolicy, PlanarDocumentInput,
    PlanarDocumentReconstructor, SurfaceGrid, apply_dense_warp, mesh_to_dense_warp,
};

fn main() {
    let image = ImageBuffer::new(160, 120, PixelFormat::Gray8, vec![220; 160 * 120])
        .expect("fixed synthetic benchmark fixture is valid");
    let input = PlanarDocumentInput {
        image,
        quad: Quad {
            points: [
                Point2::new(20.0, 10.0),
                Point2::new(140.0, 16.0),
                Point2::new(132.0, 108.0),
                Point2::new(24.0, 102.0),
            ],
        },
    };
    let mut reconstructor = PlanarDocumentReconstructor::default();
    reconstructor.config.target_long_edge = 240;
    let started = Instant::now();
    for _ in 0..3 {
        let output = reconstructor
            .reconstruct_output(&input)
            .expect("fixed synthetic benchmark must reconstruct");
        std::hint::black_box(output);
    }
    println!(
        "{}",
        BenchmarkRecord::synthetic_baseline(
            "document",
            "mp4-flat-page-perspective",
            "planar_reconstruction",
            3,
            started.elapsed().as_nanos(),
        )
        .to_json_line()
    );
    let mut margin = reconstructor.clone();
    margin.config.crop_margin_policy = CropMarginPolicy::ExpandPercent(0.04);
    margin.config.orientation_policy = OrientationPolicy::ManualRotation {
        degrees_clockwise: 90,
    };
    let started = Instant::now();
    let output = margin
        .reconstruct_output(&input)
        .expect("margin fixture must reconstruct");
    std::hint::black_box(output);
    println!(
        "{}",
        BenchmarkRecord::synthetic_baseline(
            "document",
            "mp4-rotated-margin-page",
            "planar_reconstruction_margin_orientation",
            1,
            started.elapsed().as_nanos(),
        )
        .to_json_line()
    );
    let identity = SurfaceGrid::identity(2, 2, 160, 120).expect("identity grid is valid");
    let dense = mesh_to_dense_warp(&MeshWarp { grid: identity }, 160, 120)
        .expect("identity dense field is valid");
    let started = Instant::now();
    let output = apply_dense_warp(&input.image, &dense).expect("identity remap must run");
    std::hint::black_box(output);
    println!(
        "{}",
        BenchmarkRecord::synthetic_baseline(
            "document",
            "mp4-dense-warp-identity",
            "dense_warp_identity",
            1,
            started.elapsed().as_nanos(),
        )
        .to_json_line()
    );
}
