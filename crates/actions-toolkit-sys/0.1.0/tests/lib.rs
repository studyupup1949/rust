use actions_toolkit_sys::*;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;

// #[wasm_bindgen_test]
// fn export_secret_callable() {
//     let name = <&str as Default>::default().into();
//     let value = <&str as Default>::default().into();
//     export_secret(name, value);
// }

#[wasm_bindgen_test]
fn export_variable_callable() {
    let name = <&str as Default>::default().into();
    let value = <&str as Default>::default().into();
    export_variable(name, value);
}

#[wasm_bindgen_test]
fn add_path_callable() {
    let path = <&str as Default>::default().into();
    add_path(path);
}

#[wasm_bindgen_test]
fn get_input_callable() {
    let name = <&str as Default>::default().into();
    let options = Default::default();
    get_input(name, options);
}

#[wasm_bindgen_test]
fn set_output_callable() {
    let name = <&str as Default>::default().into();
    let value = <&str as Default>::default().into();
    set_output(name, value);
}

// #[wasm_bindgen_test]
// fn set_failed_callable() {
//     let message = <&str as Default>::default().into();
//     set_failed(message);
// }

#[wasm_bindgen_test]
fn debug_callable() {
    let message = <&str as Default>::default().into();
    info(message);
}

#[wasm_bindgen_test]
fn error_callable() {
    let message = <&str as Default>::default().into();
    info(message);
}

#[wasm_bindgen_test]
fn warning_callable() {
    let message = <&str as Default>::default().into();
    info(message);
}

#[wasm_bindgen_test]
fn info_callable() {
    let message = <&str as Default>::default().into();
    info(message);
}

#[wasm_bindgen_test]
fn start_group_callable() {
    let name = <&str as Default>::default().into();
    start_group(name);
}

#[wasm_bindgen_test]
fn end_group_callable() {
    end_group();
}

#[wasm_bindgen_test]
fn group_callable() {
    let name = <&str as Default>::default().into();
    let clo = Closure::<dyn Fn()>::new(move || {});
    group(name, clo.as_ref().unchecked_ref());
    clo.forget();
}
