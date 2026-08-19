use wellfriend_perception_core::{
    ImageBuffer, PerceptionError, PixelFormat, Point2, Quad, geometry::apply_homography,
};
use wellfriend_perception_reconstruction::{
    AspectRatioPolicy, CropMarginPolicy, CurvedPageClass, CurvedPageRoute, MeshWarp,
    OrientationPolicy, PaperPreset, PlanarDocumentInput, PlanarDocumentReconstructor,
    ReconstructionContext, Reconstructor, SurfaceGrid, apply_dense_warp, mesh_to_dense_warp,
    route_curved_page,
};

fn fixture() -> PlanarDocumentInput {
    let mut bytes = vec![18; 96 * 72];
    for y in 10..62 {
        for x in 14..84 {
            bytes[y * 96 + x] = 220;
        }
    }
    PlanarDocumentInput {
        image: ImageBuffer::new(96, 72, PixelFormat::Gray8, bytes).unwrap(),
        quad: Quad {
            points: [
                Point2::new(14.0, 10.0),
                Point2::new(84.0, 13.0),
                Point2::new(80.0, 62.0),
                Point2::new(16.0, 59.0),
            ],
        },
    }
}

#[test]
fn quad_to_canonical_page_succeeds_and_preserves_projected_corners() {
    let mut reconstructor = PlanarDocumentReconstructor::default();
    reconstructor.config.target_long_edge = 160;
    let page = reconstructor.reconstruct_page(&fixture()).unwrap();
    assert!(page.geometry.width > 0 && page.geometry.height > 0);
    let projected = apply_homography(
        page.trace.transform_chain.source_to_page,
        page.trace.transform_chain.source_quad.points[0],
    )
    .unwrap();
    assert!(projected.x.abs() < 1.0 && projected.y.abs() < 1.0);
    assert!(page.quality.summary.output_quality_score.value() >= 0.0);
}

#[test]
fn invalid_quad_is_rejected() {
    let mut input = fixture();
    input.quad.points[2] = input.quad.points[1];
    assert!(matches!(
        PlanarDocumentReconstructor::default().reconstruct_page(&input),
        Err(PerceptionError::DegenerateGeometry { .. })
    ));
}

#[test]
fn known_and_manual_aspect_policies_are_respected() {
    let mut known = PlanarDocumentReconstructor::default();
    known.config.target_long_edge = 160;
    known.config.aspect_policy = AspectRatioPolicy::KnownPreset(PaperPreset::A4);
    let a4 = known.reconstruct_page(&fixture()).unwrap();
    assert!((a4.geometry.aspect_ratio - 210.0 / 297.0).abs() < 0.01);
    let mut manual = PlanarDocumentReconstructor::default();
    manual.config.target_long_edge = 160;
    manual.config.aspect_policy = AspectRatioPolicy::ManualOverride {
        width: 3.0,
        height: 2.0,
    };
    let page = manual.reconstruct_page(&fixture()).unwrap();
    assert!((page.geometry.aspect_ratio - 1.5).abs() < 0.01);
}

#[test]
fn orientation_and_margin_policies_apply_with_bounds_checks() {
    let mut reconstructor = PlanarDocumentReconstructor::default();
    reconstructor.config.target_long_edge = 160;
    reconstructor.config.aspect_policy = AspectRatioPolicy::ManualOverride {
        width: 3.0,
        height: 2.0,
    };
    reconstructor.config.orientation_policy = OrientationPolicy::LongEdgeVertical;
    reconstructor.config.crop_margin_policy = CropMarginPolicy::Manual {
        left: 4,
        top: 3,
        right: 2,
        bottom: 1,
    };
    let page = reconstructor.reconstruct_page(&fixture()).unwrap();
    assert!(page.geometry.height > page.geometry.width);
    assert!(page.geometry.width >= 6);
    reconstructor.config.crop_margin_policy = CropMarginPolicy::ExpandPercent(0.9);
    assert!(matches!(
        reconstructor.reconstruct_page(&fixture()),
        Err(PerceptionError::NumericFailure { .. })
    ));
}

#[test]
fn reconstruct_trait_wraps_canonical_document() {
    let mut reconstructor = PlanarDocumentReconstructor::default();
    reconstructor.config.target_long_edge = 160;
    let document = reconstructor
        .reconstruct(&fixture(), &ReconstructionContext::default())
        .unwrap();
    assert_eq!(document.pages.len(), 1);
}

#[test]
fn surface_grid_and_dense_identity_warp_are_checked() {
    let image = ImageBuffer::new(5, 4, PixelFormat::Gray8, (0..20).collect()).unwrap();
    let grid = SurfaceGrid::identity(2, 2, 5, 4).unwrap();
    let field = mesh_to_dense_warp(&MeshWarp { grid }, 5, 4).unwrap();
    assert_eq!(apply_dense_warp(&image, &field).unwrap(), image);
}

#[test]
fn invalid_surface_and_curved_routes_are_explicit() {
    let invalid = SurfaceGrid {
        columns: 1,
        rows: 1,
        control_points: Vec::new(),
    };
    assert!(invalid.validate().is_err());
    assert!(matches!(
        route_curved_page(CurvedPageClass::FlatPage),
        CurvedPageRoute::Planar
    ));
    assert!(matches!(
        route_curved_page(CurvedPageClass::StrongCurvature),
        CurvedPageRoute::SurfaceUnavailable { .. }
    ));
}
