use js_sys::{Array, JsString, Object, Promise};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "@actions/exec")]
extern {
    // FIXME: switch to typed options; requires node typings
    /// Execute a command and stream the output to the console.
    #[wasm_bindgen]
    pub fn exec(command: &JsString, args: Option<&Array>, options: Option<&Object>) -> Promise;
}
