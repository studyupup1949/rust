use actions_toolkit_sys::core;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;

// #[wasm_bindgen_test]
// fn export_secret_callable() {
//     let name = &"name".into();
//     let value = &"value".into();
//     core::export_secret(name, value);
// }

#[wasm_bindgen_test]
fn export_variable_callable() {
    let name = &"name".into();
    let value = &"value".into();
    core::export_variable(name, value);
}

#[wasm_bindgen_test]
fn add_path_callable() {
    let path = &"path".into();
    core::add_path(path);
}

#[wasm_bindgen_test]
fn get_input_callable() {
    let name = &"name".into();
    let options = Default::default();
    core::get_input(name, options);
}

#[wasm_bindgen_test]
fn set_output_callable() {
    let name = &"name".into();
    let value = &"value".into();
    core::set_output(name, value);
}

// #[wasm_bindgen_test]
// fn set_failed_callable() {
//     let message = "message".into();
//     core::set_failed(message);
// }

#[wasm_bindgen_test]
fn debug_callable() {
    let message = &"message".into();
    core::info(message);
}

#[wasm_bindgen_test]
fn error_callable() {
    let message = &"message".into();
    core::info(message);
}

#[wasm_bindgen_test]
fn warning_callable() {
    let message = &"message".into();
    core::info(message);
}

#[wasm_bindgen_test]
fn info_callable() {
    let message = &"message".into();
    core::info(message);
}

#[wasm_bindgen_test]
fn start_group_callable() {
    let name = &"name".into();
    core::start_group(name);
}

#[wasm_bindgen_test]
fn end_group_callable() {
    core::end_group();
}

#[wasm_bindgen_test]
fn group_callable() {
    let name = &"name".into();
    let clo = Closure::wrap(Box::new(|| {}) as Box<dyn Fn()>);
    core::group(name, clo.as_ref().unchecked_ref());
    clo.forget();
}
