use js_sys::{Array, Error, JsString, Number, Promise};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "@actions/tool-cache")]
extern {
    #[wasm_bindgen(extends = Error)]
    pub type HTTPError;

    #[wasm_bindgen(method, getter, js_name = "httpStatusCode")]
    pub fn http_status_code(this: &HTTPError) -> Option<Number>;

    #[wasm_bindgen(constructor)]
    pub fn new(http_status_code: Option<&Number>) -> HTTPError;
}

#[wasm_bindgen(module = "@actions/tool-cache")]
extern {
    /// Download a tool from an url and stream it into a file.
    #[wasm_bindgen(js_name = "downloadTool")]
    #[must_use] // Promise<string>
    pub fn download_tool(url: &JsString) -> Promise;

    /// Extract a .7z file.
    #[wasm_bindgen(js_name = "extract7z")]
    #[must_use] // Promise<string>
    pub fn extract_7z(file: &JsString, dest: Option<&JsString>, _7z_path: Option<&JsString>) -> Promise;

    /// Extract a tar.
    #[wasm_bindgen(js_name = "extractTar")]
    #[must_use] // Promise<string>
    pub fn extract_tar(file: &JsString, dest: Option<&JsString>, flags: Option<&JsString>) -> Promise;

    /// Extract a zip.
    #[wasm_bindgen(js_name = "extractZip")]
    #[must_use] // Promise<string>
    pub fn extract_zip(file: &JsString, dest: Option<&JsString>) -> Promise;

    /// Caches a directory and installs it into the tool cacheDir.
    #[wasm_bindgen(js_name = "cacheDir")]
    #[must_use] // Promise<string>
    pub fn cache_dir(source: &JsString, tool: &JsString, version: &JsString, arch: Option<&JsString>) -> Promise;

    /// Caches a downloaded file (GUID) and installs it into the tool cache with a given targetName.
    #[wasm_bindgen(js_name = "cacheFile")]
    #[must_use] // Promise<string>
    pub fn cache_file(
        source: &JsString,
        target: &JsString,
        tool: &JsString,
        version: &JsString,
        arch: Option<&JsString>,
    ) -> Promise;

    /// Finds the path to a tool version in the local installed tool cache.
    #[wasm_bindgen]
    pub fn find(tool: &JsString, version: &JsString, arch: Option<&JsString>) -> String;

    /// Finds the paths to all versions of a tool that are installed in the local tool cache.
    #[wasm_bindgen(js_name = "findAllVersions")]
    pub fn find_all_versions(tool: &JsString, arch: Option<&JsString>) -> Array;
}
