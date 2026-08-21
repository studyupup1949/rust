use actions_toolkit_sys::tool_cache;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn download_tool_callable() {
    let url = &"url".into();
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    tool_cache::download_tool(url).catch(&clo);
    clo.forget();
}

#[wasm_bindgen_test]
fn extract_7z_callable() {
    let file = &"file".into();
    let dest = Default::default();
    #[allow(non_snake_case)]
    let _7z_path = Default::default();
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    tool_cache::extract_7z(file, dest, _7z_path).catch(&clo);
    clo.forget();
}

#[wasm_bindgen_test]
fn extract_tar_callable() {
    let file = &"url".into();
    let dest = Default::default();
    let flags = Default::default();
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    tool_cache::extract_tar(file, dest, flags).catch(&clo);
    clo.forget();
}

#[wasm_bindgen_test]
fn extract_zip_callable() {
    let file = &"file".into();
    let dest = Default::default();
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    tool_cache::extract_zip(file, dest).catch(&clo);
    clo.forget();
}

#[wasm_bindgen_test]
fn cache_dir_callable() {
    let source = &"source".into();
    let tool = &"tool".into();
    let version = &"version".into();
    let arch = Default::default();
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    tool_cache::cache_dir(source, tool, version, arch).catch(&clo);
    clo.forget();
}

#[wasm_bindgen_test]
fn cache_file_callable() {
    let source = &"source".into();
    let target = &"target".into();
    let tool = &"tool".into();
    let version = &"version".into();
    let arch = Default::default();
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    tool_cache::cache_file(source, target, tool, version, arch).catch(&clo);
    clo.forget();
}

#[wasm_bindgen_test]
fn find_callable() {
    let tool = &"tool".into();
    let version = &"version".into();
    let arch = Default::default();
    tool_cache::find(tool, version, arch);
}

#[wasm_bindgen_test]
fn find_all_versions_callable() {
    let tool = &"tool".into();
    let arch = Default::default();
    tool_cache::find_all_versions(tool, arch);
}
