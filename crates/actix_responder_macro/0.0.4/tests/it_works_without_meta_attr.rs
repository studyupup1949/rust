extern crate actix_responder_macro;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use serde::{Deserialize, Serialize};

#[actix_responder]
#[derive(Serialize, Deserialize, Debug, Default)]
struct SuccessResp {
    success: bool,
}

#[test]
fn it_works_without_meta_attr() {
    SuccessResp::default();
}
