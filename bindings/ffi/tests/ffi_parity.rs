use std::ffi::{CStr, CString};

use wellfriend_perception_ffi::{
    runtime::{AnalyzeFrameResponse, ApplyFilterResponse, ReconstructPageResponse, RuntimeEngine},
    wf_analyze_frame, wf_apply_filter, wf_engine_create, wf_engine_destroy, wf_reconstruct_page,
    wf_string_free,
};

fn fixture() -> Vec<u8> {
    let mut image = vec![16; 80 * 60];
    for y in 10..50 {
        for x in 16..64 {
            image[y * 80 + x] = 240;
        }
    }
    image
}

unsafe fn owned_json(pointer: *mut std::ffi::c_char) -> String {
    // SAFETY: test only passes a returned owned runtime string and frees it exactly once.
    let output = unsafe { CStr::from_ptr(pointer) }
        .to_str()
        .unwrap()
        .to_owned();
    // SAFETY: pointer comes from a Wellfriend FFI function.
    unsafe { wf_string_free(pointer) };
    output
}

#[test]
fn c_abi_matches_direct_scalar_guidance_on_golden_fixture() {
    let bytes = fixture();
    let direct = RuntimeEngine::new("{}")
        .unwrap()
        .analyze(&bytes, 80, 60, 80, "Gray8", "{}")
        .unwrap();
    let direct: AnalyzeFrameResponse = serde_json::from_str(&direct).unwrap();
    let config = CString::new("{}").unwrap();
    // SAFETY: test passes valid NUL-terminated config and keeps it alive during the call.
    let engine = unsafe { wf_engine_create(config.as_ptr()) };
    assert!(!engine.is_null());
    let format = CString::new("Gray8").unwrap();
    let request = CString::new("{}").unwrap();
    // SAFETY: valid engine, image buffer, dimensions, stride, and NUL-terminated strings.
    let response = unsafe {
        wf_analyze_frame(
            engine,
            bytes.as_ptr(),
            80,
            60,
            80,
            format.as_ptr(),
            request.as_ptr(),
        )
    };
    let response: AnalyzeFrameResponse =
        serde_json::from_str(&unsafe { owned_json(response) }).unwrap();
    // SAFETY: this is the sole destruction of an engine returned above.
    unsafe { wf_engine_destroy(engine) };

    assert_eq!(response.schema_version, direct.schema_version);
    assert_eq!(response.capture_readiness, direct.capture_readiness);
    assert_eq!(
        response.boundary.geometry.is_some(),
        direct.boundary.geometry.is_some()
    );
    assert_eq!(
        response.image_free_candidate_count(),
        direct.image_free_candidate_count()
    );
}

trait CandidateCount {
    fn image_free_candidate_count(&self) -> usize;
}
impl CandidateCount for AnalyzeFrameResponse {
    fn image_free_candidate_count(&self) -> usize {
        self.candidates.len()
    }
}

#[test]
fn c_abi_rejects_invalid_stride_with_structured_error() {
    let config = CString::new("{}").unwrap();
    // SAFETY: valid configuration string.
    let engine = unsafe { wf_engine_create(config.as_ptr()) };
    let bytes = fixture();
    let format = CString::new("Gray8").unwrap();
    let request = CString::new("{}").unwrap();
    // SAFETY: deliberately invalid stride is validated before image construction.
    let response = unsafe {
        wf_analyze_frame(
            engine,
            bytes.as_ptr(),
            80,
            60,
            1,
            format.as_ptr(),
            request.as_ptr(),
        )
    };
    let response = unsafe { owned_json(response) };
    // SAFETY: sole destruction of a live test engine.
    unsafe { wf_engine_destroy(engine) };
    assert!(response.contains("error"));
}

#[test]
fn c_abi_runs_real_reconstruction_and_filter_paths() {
    let bytes = fixture();
    let config = CString::new("{}").unwrap();
    // SAFETY: valid configuration string.
    let engine = unsafe { wf_engine_create(config.as_ptr()) };
    let format = CString::new("Gray8").unwrap();
    let reconstruct = CString::new(
        r#"{"quad":{"points":[{"x":16.0,"y":10.0},{"x":63.0,"y":10.0},{"x":63.0,"y":49.0},{"x":16.0,"y":49.0}]},"output_long_edge":256}"#,
    )
    .unwrap();
    // SAFETY: valid engine, bytes, geometry, dimensions, and C strings.
    let reconstructed = unsafe {
        wf_reconstruct_page(
            engine,
            bytes.as_ptr(),
            80,
            60,
            80,
            format.as_ptr(),
            reconstruct.as_ptr(),
        )
    };
    let reconstructed: ReconstructPageResponse =
        serde_json::from_str(&unsafe { owned_json(reconstructed) }).unwrap();
    assert_eq!(reconstructed.image.pixel_format, "Gray8");
    assert!(reconstructed.image.width >= 256 || reconstructed.image.height >= 256);

    let filter = CString::new(r#"{"preset":"Grayscale"}"#).unwrap();
    // SAFETY: valid engine, bytes, dimensions, and C strings.
    let filtered = unsafe {
        wf_apply_filter(
            engine,
            bytes.as_ptr(),
            80,
            60,
            80,
            format.as_ptr(),
            filter.as_ptr(),
        )
    };
    let filtered: ApplyFilterResponse =
        serde_json::from_str(&unsafe { owned_json(filtered) }).unwrap();
    // SAFETY: sole destruction of a live test engine.
    unsafe { wf_engine_destroy(engine) };
    assert_eq!(filtered.image.pixel_format, "Gray8");
}
