use js_sys::{JsString, Promise};
use wasm_bindgen::prelude::*;

/// Interface for cp options.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CopyOptions {
    /// Whether to recursively copy all subdirectories. Defaults to false.
    pub recursive: Option<bool>,
    /// Whether to overwrite existing files in the destination. Defaults to true.
    pub force: Option<bool>,
}

/// Interface for mv options.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MoveOptions {
    /// Whether to overwrite existing files in the destination. Defaults to true.
    pub force: Option<bool>,
}

#[wasm_bindgen(module = "@actions/io")]
extern {
    /// Copies a file or folder.
    #[wasm_bindgen]
    #[must_use]
    pub fn cp(source: &JsString, target: &JsString, options: Option<CopyOptions>) -> Promise;

    /// Moves a path.
    #[wasm_bindgen]
    #[must_use]
    pub fn mv(source: &JsString, target: &JsString, options: Option<MoveOptions>) -> Promise;

    /// Remove a path recursively with force.
    #[wasm_bindgen(js_name = "rmRF")]
    #[must_use]
    pub fn rm_rf(path: &JsString) -> Promise;

    /// Make a directory.  Creates the full path with folders in between. Will throw if it fails.
    #[wasm_bindgen(js_name = "mkdirP")]
    #[must_use]
    pub fn mkdir_p(path: &JsString) -> Promise;

    /// Returns path of a tool had the tool actually been invoked.  Resolves via paths. If you check
    /// and the tool does not exist, it will throw.
    #[wasm_bindgen]
    #[must_use]
    pub fn which(tool: &JsString, check: Option<bool>) -> Promise;
}
