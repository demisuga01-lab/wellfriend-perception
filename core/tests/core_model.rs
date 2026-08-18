use wellfriend_perception_core::*;

#[test]
fn image_shape_stride_and_roi_are_validated() {
    assert!(matches!(
        ImageShape::new(0, 2),
        Err(PerceptionError::InvalidDimensions { .. })
    ));
    let shape = ImageShape::new(2, 2).unwrap();
    assert!(matches!(
        ImageBuffer::new_with_stride(shape, PixelFormat::Gray8, Stride(1), vec![0; 2]),
        Err(PerceptionError::StrideMismatch { .. })
    ));
    let image = ImageBuffer::new_with_stride(
        shape,
        PixelFormat::Gray8,
        Stride(3),
        vec![1, 2, 99, 3, 4, 99],
    )
    .unwrap();
    assert_eq!(image.view().row(1).unwrap(), &[3, 4]);
    let roi = RegionOfInterest::within(shape, 1, 0, 1, 2).unwrap();
    assert_eq!(
        image.roi(roi).unwrap().to_owned().unwrap().as_bytes(),
        &[2, 4]
    );
    assert!(RegionOfInterest::within(shape, 2, 0, 1, 1).is_err());
}

#[test]
fn mutable_view_and_f32_buffers_are_safe() {
    let mut image = ImageBuffer::new(2, 1, PixelFormat::Gray8, vec![1, 2]).unwrap();
    image.view_mut().row_mut(0).unwrap()[1] = 8;
    assert_eq!(image.get_u8(1, 0, 0).unwrap(), 8);
    let floats = ImageBuffer::from_f32(1, 2, PixelFormat::GrayF32, vec![0.25, 0.75]).unwrap();
    assert_eq!(floats.to_f32().unwrap(), vec![0.25, 0.75]);
}

#[test]
fn confidence_rejects_invalid_values() {
    for value in [-0.1, 1.1, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(matches!(
            Confidence::new(value),
            Err(PerceptionError::InvalidConfidence { .. })
        ));
    }
    assert_eq!(Confidence::new(0.92).unwrap().value(), 0.92);
}

#[test]
fn diagnostics_collect_stage_events() {
    let mut trace = PipelineTrace::default();
    trace.push(Diagnostic {
        level: DiagnosticLevel::Info,
        code: DiagnosticCode("CORE_READY".into()),
        message: "ready".into(),
    });
    trace.record_timing(StageTiming {
        stage: PipelineStage::Input,
        started_at_us: 100,
        duration_us: 25,
    });
    assert_eq!(trace.diagnostics.len(), 1);
    assert_eq!(trace.timings[0].duration_us, 25);
}
