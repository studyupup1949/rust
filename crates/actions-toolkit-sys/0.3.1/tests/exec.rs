use actions_toolkit_sys::exec;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn exec_callable() {
    let command = &"command".into();
    let args = Default::default();
    let options = Default::default();
    exec::exec(command, args, options);
}
