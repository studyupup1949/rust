use js_sys::{JsString, Object};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "@actions/core")]
extern {
    #[wasm_bindgen(js_name = "issue")]
    pub fn issue(name: JsString, message: Option<JsString>);

    #[wasm_bindgen(js_name = "issueCommand")]
    pub fn issue_command(command: JsString, properties: &Object, message: JsString);
}
