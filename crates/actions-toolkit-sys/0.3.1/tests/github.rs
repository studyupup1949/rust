use actions_toolkit_sys::github::GitHub;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn new_github_callable() {
    let token = &"token".into();
    let options = None;
    GitHub::new(token, options);
}

#[wasm_bindgen_test]
fn graphql_callable() {
    let token = &"token".into();
    let options = None;
    let kit = GitHub::new(token, options);
    let query = &"query".into();
    let variables = None;
    let clo = Closure::wrap(Box::new(|_err| {}) as Box<_>);
    kit.graphql(query, variables).catch(&clo);
    clo.forget();
}
