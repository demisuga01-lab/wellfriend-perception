//! Browser-facing WebAssembly bridge for the Wellfriend scalar runtime.
//!
//! Calls return JSON matching the C ABI's stable runtime schema.  No browser
//! fallback runs a TypeScript detector: absence or loading failure of this module
//! is an explicit production error handled by `wellfriend-scan`.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wellfriend_perception::runtime::{RuntimeEngine, runtime_error_json};

/// Opaque WASM runtime handle. It owns no image data between calls.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct EngineHandle {
    #[cfg(target_arch = "wasm32")]
    engine: RuntimeEngine,
}

#[cfg(target_arch = "wasm32")]
fn as_js_error(error: String) -> JsValue {
    JsValue::from_str(&runtime_error_json("runtime_error", &error))
}

/// Creates one configured scalar engine.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = createEngine))]
#[cfg(target_arch = "wasm32")]
pub fn create_engine(config_json: Option<String>) -> Result<EngineHandle, JsValue> {
    Ok(EngineHandle {
        engine: RuntimeEngine::new(config_json.as_deref().unwrap_or("{}")).map_err(as_js_error)?,
    })
}

/// Returns the scalar runtime's stable semantic version.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = version))]
#[cfg(target_arch = "wasm32")]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").into()
}

/// Destroys a handle explicitly. Rust would also drop it when JavaScript releases it.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = destroyEngine))]
#[cfg(target_arch = "wasm32")]
pub fn destroy_engine(_handle: EngineHandle) {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl EngineHandle {
    /// Runs real scalar quality, document detection, fusion, refinement, and readiness.
    #[wasm_bindgen(js_name = analyzeFrame)]
    pub fn analyze_frame(
        &self,
        image: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        pixel_format: String,
        request_json: String,
    ) -> Result<String, JsValue> {
        self.engine
            .analyze(image, width, height, stride, &pixel_format, &request_json)
            .map_err(as_js_error)
    }

    /// Runs real scalar planar reconstruction for a selected, validated quad.
    #[wasm_bindgen(js_name = reconstructPage)]
    pub fn reconstruct_page(
        &self,
        image: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        pixel_format: String,
        request_json: String,
    ) -> Result<String, JsValue> {
        self.engine
            .reconstruct(image, width, height, stride, &pixel_format, &request_json)
            .map_err(as_js_error)
    }

    /// Runs real scalar filter processing for implemented presets.
    #[wasm_bindgen(js_name = applyFilter)]
    pub fn apply_filter(
        &self,
        image: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        pixel_format: String,
        request_json: String,
    ) -> Result<String, JsValue> {
        self.engine
            .apply_filter(image, width, height, stride, &pixel_format, &request_json)
            .map_err(as_js_error)
    }
}

/// Native-only marker that lets workspace CI validate this package before a
/// `wasm32-unknown-unknown` build is requested.
#[cfg(not(target_arch = "wasm32"))]
pub const WASM_TARGET_REQUIRED: &str = "build this crate with wasm32-unknown-unknown";
