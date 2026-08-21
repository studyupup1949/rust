use actions_toolkit_sys::io;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

// FIXME: Should we log the errors from the catches?

#[wasm_bindgen_test]
fn cp_callable() {
    let source = &"source".into();
    let target = &"target".into();
    let options = None;
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    io::cp(source, target, options).catch(&clo);
    clo.forget()
}

#[wasm_bindgen_test]
fn mv_callable() {
    let source = &"source".into();
    let target = &"target".into();
    let options = None;
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    io::mv(source, target, options).catch(&clo);
    clo.forget()
}

#[wasm_bindgen_test]
fn rm_rf() {
    let path = &"path".into();
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    io::rm_rf(path).catch(&clo);
    clo.forget()
}

#[wasm_bindgen_test]
fn mkdir_p() {
    let path = &"path".into();
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    io::mkdir_p(path).catch(&clo);
    clo.forget()
}

#[wasm_bindgen_test]
fn which() {
    let tool = &"tool".into();
    let check = None;
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    io::which(tool, check).catch(&clo);
    clo.forget()
}
