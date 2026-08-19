use wellfriend_perception_core::{geometry::*, math::*, *};

fn close(left: f32, right: f32) {
    assert!((left - right).abs() < 1.0e-4, "{left} != {right}");
}

#[test]
fn vector_matrix_and_statistics_primitives_are_correct() {
    let a = Vec2::new(3.0, 4.0);
    close(a.norm(), 5.0);
    close(a.dot(Vec2::new(2.0, 1.0)), 10.0);
    close(a.cross(Vec2::new(2.0, 1.0)), -5.0);
    assert_eq!(
        Vec3::new(1.0, 0.0, 0.0).cross(Vec3::new(0.0, 1.0, 0.0)),
        Vec3::new(0.0, 0.0, 1.0)
    );
    let matrix = Mat3::new([[2.0, 0.0, 3.0], [0.0, 4.0, -2.0], [0.0, 0.0, 1.0]]);
    let product = matrix * matrix.inverse().unwrap();
    for row in 0..3 {
        for column in 0..3 {
            close(
                product.values[row][column],
                if row == column { 1.0 } else { 0.0 },
            );
        }
    }
    assert_eq!(Mat4::identity().inverse().unwrap(), Mat4::identity());
    close(mean(&[1.0, 2.0, 3.0]).unwrap(), 2.0);
    close(median(&[1.0, 9.0, 3.0, 5.0]).unwrap(), 4.0);
    close(percentile(&[0.0, 10.0], 0.25).unwrap(), 2.5);
}

#[test]
fn polygons_lines_and_quad_validation_are_correct() {
    let polygon = Polygon {
        points: vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 2.0),
            Point2::new(0.0, 2.0),
        ],
    };
    close(polygon.area(), 8.0);
    assert_eq!(polygon.winding(), Winding::CounterClockwise);
    assert!(polygon.contains_point(Point2::new(2.0, 1.0)));
    assert!(!polygon.contains_point(Point2::new(5.0, 1.0)));
    assert_eq!(polygon.centroid().unwrap(), Point2::new(2.0, 1.0));
    let intersection = line_intersection(
        Line2 {
            a: Point2::new(0.0, 0.0),
            b: Point2::new(2.0, 2.0),
        },
        Line2 {
            a: Point2::new(0.0, 2.0),
            b: Point2::new(2.0, 0.0),
        },
    )
    .unwrap();
    assert_eq!(intersection, Point2::new(1.0, 1.0));
    assert_eq!(
        segment_intersection(
            Segment2 {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(1.0, 0.0)
            },
            Segment2 {
                start: Point2::new(2.0, 0.0),
                end: Point2::new(3.0, 0.0)
            }
        ),
        None
    );
    Quad {
        points: [
            Point2::new(0.0, 0.0),
            Point2::new(2.0, 0.0),
            Point2::new(2.0, 1.0),
            Point2::new(0.0, 1.0),
        ],
    }
    .validate()
    .unwrap();
}

#[test]
fn transforms_and_homographies_round_trip_generated_points() {
    let transform = Transform2D::translation(4.0, -3.0)
        .compose(Transform2D::rotation(0.2))
        .compose(Transform2D::scale(1.5, 0.75));
    for x in -3..=3 {
        for y in -3..=3 {
            let original = Point2::new(x as f32 * 0.25, y as f32 * 0.5);
            let recovered = transform
                .inverse()
                .unwrap()
                .apply_point(transform.apply_point(original).unwrap())
                .unwrap();
            close(recovered.x, original.x);
            close(recovered.y, original.y);
        }
    }
    let source = [
        Point2::new(0.0, 0.0),
        Point2::new(2.0, 0.0),
        Point2::new(2.0, 1.0),
        Point2::new(0.0, 1.0),
    ];
    let target = [
        Point2::new(1.0, 2.0),
        Point2::new(5.0, 1.0),
        Point2::new(4.0, 4.0),
        Point2::new(0.0, 5.0),
    ];
    let homography = estimate_homography_4pt(source, target).unwrap();
    for (from, expected) in source.into_iter().zip(target) {
        let actual = apply_homography(homography, from).unwrap();
        close(actual.x, expected.x);
        close(actual.y, expected.y);
        let round_trip = invert_homography(homography)
            .unwrap()
            .apply_point(actual)
            .unwrap();
        close(round_trip.x, from.x);
        close(round_trip.y, from.y);
    }
}

#[test]
fn warp_remap_and_line_fitting_handle_baseline_cases() {
    let image = ImageBuffer::new(2, 2, PixelFormat::Gray8, vec![1, 2, 3, 4]).unwrap();
    let warped = warp_perspective(
        &image,
        Transform2D::identity(),
        ImageShape::new(2, 2).unwrap(),
        SamplingMode::Nearest,
        WarpBorder::Constant(0),
    )
    .unwrap();
    assert_eq!(warped.as_bytes(), image.as_bytes());
    let remapped = remap(
        &image,
        &DenseWarpField {
            width: 2,
            height: 2,
            vectors: vec![
                Point2::new(0.0, 0.0),
                Point2::new(1.0, 0.0),
                Point2::new(0.0, 1.0),
                Point2::new(1.0, 1.0),
            ],
        },
        SamplingMode::Bilinear,
        WarpBorder::Replicate,
    )
    .unwrap();
    assert_eq!(remapped.as_bytes(), image.as_bytes());
    let points = [
        Point2::new(0.0, 1.0),
        Point2::new(1.0, 3.0),
        Point2::new(2.0, 5.0),
        Point2::new(3.0, 7.0),
    ];
    assert!(least_squares_line_fit(&points).unwrap().rms_error < 1.0e-4);
    let with_outlier = [
        points[0],
        points[1],
        points[2],
        points[3],
        Point2::new(1.0, -10.0),
    ];
    let robust = ransac_line_fit(
        &with_outlier,
        RansacConfig {
            iterations: 32,
            inlier_threshold: 0.1,
            seed: 7,
        },
    )
    .unwrap();
    assert_eq!(robust.inliers.iter().filter(|inside| **inside).count(), 4);
}
