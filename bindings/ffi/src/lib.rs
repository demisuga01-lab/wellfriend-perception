//! Audited C ABI boundary for the scalar Wellfriend perception runtime.
//!
//! The exported functions use raw bytes plus JSON to keep ownership explicit at
//! the first Android and desktop bridge.  All pointer dereferences are confined
//! to this module; callers must pass valid NUL-terminated strings and buffers of
//! the declared `stride * height` length.  Errors are returned as JSON strings
//! and every returned allocation is released by [`wf_string_free`].

#![allow(unsafe_code)]

pub mod runtime;

use std::{
    ffi::{CStr, CString, c_char},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr, slice,
    sync::Mutex,
};

use runtime::{RuntimeEngine, runtime_error_json};

const MAX_JSON_BYTES: usize = 256 * 1024;
static EMPTY_C_STRING: [u8; 1] = [0];

/// Opaque native runtime state. Its last error pointer remains valid until the
/// next call on this engine or destruction.
pub struct WfEngine {
    runtime: RuntimeEngine,
    last_error: Mutex<Option<CString>>,
}

fn string_result(result: Result<String, String>, engine: *mut WfEngine) -> *mut c_char {
    match result {
        Ok(json) => into_c_string(json),
        Err(error) => {
            // SAFETY: the public callers have already checked that engine is non-null.
            unsafe { set_last_error(engine, &error) };
            into_c_string(runtime_error_json("runtime_error", &error))
        }
    }
}

fn into_c_string(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(value) => value.into_raw(),
        Err(_) => CString::new("{\"schema_version\":1,\"error\":{\"code\":\"encoding_error\"}}")
            .expect("static JSON contains no NUL")
            .into_raw(),
    }
}

unsafe fn set_last_error(engine: *mut WfEngine, error: &str) {
    // SAFETY: `engine` is checked by public callers and points to WfEngine while alive.
    let engine = unsafe { &*engine };
    if let Ok(mut last_error) = engine.last_error.lock() {
        *last_error = CString::new(error).ok();
    }
}

unsafe fn read_optional_json(pointer: *const c_char, name: &str) -> Result<String, String> {
    if pointer.is_null() {
        return Ok("{}".into());
    }
    // SAFETY: C ABI contract requires a valid NUL-terminated string. It is copied immediately.
    let bytes = unsafe { CStr::from_ptr(pointer) }.to_bytes();
    if bytes.len() > MAX_JSON_BYTES {
        return Err(format!("{name} exceeds {MAX_JSON_BYTES} bytes"));
    }
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| format!("{name} must be UTF-8"))
}

unsafe fn read_required_text(pointer: *const c_char, name: &str) -> Result<String, String> {
    if pointer.is_null() {
        return Err(format!("{name} must not be null"));
    }
    // SAFETY: C ABI contract requires a valid NUL-terminated string. It is copied immediately.
    let bytes = unsafe { CStr::from_ptr(pointer) }.to_bytes();
    if bytes.len() > 64 {
        return Err(format!("{name} exceeds 64 bytes"));
    }
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| format!("{name} must be UTF-8"))
}

unsafe fn read_image(
    image_bytes: *const u8,
    width: u32,
    height: u32,
    stride: u32,
    pixel_format: *const c_char,
) -> Result<(Vec<u8>, String), String> {
    if image_bytes.is_null() {
        return Err("image_bytes must not be null".into());
    }
    let format = unsafe { read_required_text(pixel_format, "pixel_format") }?;
    let length = (stride as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| "image length overflow".to_string())?;
    if width == 0 || height == 0 || stride == 0 {
        return Err("image width, height, and stride must be non-zero".into());
    }
    // SAFETY: caller contract supplies an allocated readable buffer of exactly at least length bytes.
    let bytes = unsafe { slice::from_raw_parts(image_bytes, length) }.to_vec();
    Ok((bytes, format))
}

/// Creates an engine after validating an optional JSON configuration object.
///
/// # Safety
/// `config_json` may be null; otherwise it must be a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wf_engine_create(config_json: *const c_char) -> *mut WfEngine {
    match catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: documented C input contract.
        let config = unsafe { read_optional_json(config_json, "config_json") }?;
        let runtime = RuntimeEngine::new(&config)?;
        Ok::<_, String>(Box::into_raw(Box::new(WfEngine {
            runtime,
            last_error: Mutex::new(None),
        })))
    })) {
        Ok(Ok(engine)) => engine,
        _ => ptr::null_mut(),
    }
}

/// Destroys an engine. A null pointer is accepted as a no-op.
///
/// # Safety
/// `engine` must be null or a pointer returned by [`wf_engine_create`] not previously destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wf_engine_destroy(engine: *mut WfEngine) {
    if !engine.is_null() {
        // SAFETY: documented ownership transfer back from `wf_engine_create`.
        unsafe { drop(Box::from_raw(engine)) };
    }
}

/// Runs the real scalar quality, document detection, fusion, refinement, and readiness path.
///
/// # Safety
/// All pointers must satisfy the C ABI contract described in this crate's module documentation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wf_analyze_frame(
    engine: *mut WfEngine,
    image_bytes: *const u8,
    width: u32,
    height: u32,
    stride: u32,
    pixel_format: *const c_char,
    request_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() {
        return into_c_string(runtime_error_json(
            "invalid_engine",
            "engine must not be null",
        ));
    }
    match catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: documented C input contract.
        let (bytes, format) =
            unsafe { read_image(image_bytes, width, height, stride, pixel_format) }?;
        // SAFETY: documented C input contract.
        let request = unsafe { read_optional_json(request_json, "request_json") }?;
        // SAFETY: engine is checked above and remains caller-owned for this call.
        let runtime = unsafe { &(*engine).runtime };
        runtime.analyze(&bytes, width, height, stride, &format, &request)
    })) {
        Ok(result) => string_result(result, engine),
        Err(_) => string_result(Err("panic contained at FFI boundary".into()), engine),
    }
}

/// Runs real scalar planar reconstruction for a validated selected quad.
///
/// # Safety
/// All pointers must satisfy the C ABI contract described in this crate's module documentation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wf_reconstruct_page(
    engine: *mut WfEngine,
    image_bytes: *const u8,
    width: u32,
    height: u32,
    stride: u32,
    pixel_format: *const c_char,
    request_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() {
        return into_c_string(runtime_error_json(
            "invalid_engine",
            "engine must not be null",
        ));
    }
    match catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: documented C input contract.
        let (bytes, format) =
            unsafe { read_image(image_bytes, width, height, stride, pixel_format) }?;
        // SAFETY: documented C input contract.
        let request = unsafe { read_optional_json(request_json, "request_json") }?;
        // SAFETY: engine is checked above and remains caller-owned for this call.
        unsafe { &(*engine).runtime }.reconstruct(&bytes, width, height, stride, &format, &request)
    })) {
        Ok(result) => string_result(result, engine),
        Err(_) => string_result(Err("panic contained at FFI boundary".into()), engine),
    }
}

/// Applies a real scalar restoration/filter plan for implemented presets.
///
/// # Safety
/// All pointers must satisfy the C ABI contract described in this crate's module documentation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wf_apply_filter(
    engine: *mut WfEngine,
    image_bytes: *const u8,
    width: u32,
    height: u32,
    stride: u32,
    pixel_format: *const c_char,
    request_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() {
        return into_c_string(runtime_error_json(
            "invalid_engine",
            "engine must not be null",
        ));
    }
    match catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: documented C input contract.
        let (bytes, format) =
            unsafe { read_image(image_bytes, width, height, stride, pixel_format) }?;
        // SAFETY: documented C input contract.
        let request = unsafe { read_optional_json(request_json, "request_json") }?;
        // SAFETY: engine is checked above and remains caller-owned for this call.
        unsafe { &(*engine).runtime }.apply_filter(&bytes, width, height, stride, &format, &request)
    })) {
        Ok(result) => string_result(result, engine),
        Err(_) => string_result(Err("panic contained at FFI boundary".into()), engine),
    }
}

/// Releases a string returned by an FFI operation.
///
/// # Safety
/// `pointer` must be null or an unmodified pointer returned by this library and not previously freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wf_string_free(pointer: *mut c_char) {
    if !pointer.is_null() {
        // SAFETY: documented allocation provenance and one-time ownership transfer.
        unsafe { drop(CString::from_raw(pointer)) };
    }
}

/// Returns the last engine-specific error. It remains owned by the engine.
///
/// # Safety
/// `engine` must be null or an active pointer returned by [`wf_engine_create`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wf_last_error(engine: *const WfEngine) -> *const c_char {
    if engine.is_null() {
        return EMPTY_C_STRING.as_ptr().cast();
    }
    // SAFETY: documented active engine pointer.
    let engine = unsafe { &*engine };
    match engine.last_error.lock() {
        Ok(error) => error
            .as_ref()
            .map_or(EMPTY_C_STRING.as_ptr().cast(), |value| value.as_ptr()),
        Err(_) => EMPTY_C_STRING.as_ptr().cast(),
    }
}

/// Returns the semantic runtime version without allocating.
#[unsafe(no_mangle)]
pub extern "C" fn wf_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}
