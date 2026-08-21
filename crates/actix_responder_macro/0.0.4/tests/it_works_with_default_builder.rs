extern crate actix_responder_macro;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[actix_responder(meta_attr = "builder(default)")]
#[derive(Serialize, Deserialize, Debug, TypedBuilder, Default, Eq, PartialEq)]
struct SuccessResp {
    success: bool,
}

#[test]
fn it_works_with_default_builder() {
    let actual = SuccessResp::builder()
        .success(true)
        .status_code(StatusCode::IM_A_TEAPOT)
        .build();

    let expected = SuccessResp {
        success: true,
        content_type: Default::default(),
        status_code: StatusCode::IM_A_TEAPOT,
    };

    assert_eq!(actual, expected)
}
