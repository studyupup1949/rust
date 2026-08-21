extern crate actix_responder_macro;
extern crate mime;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[actix_responder(
    status_attr = "builder(default = StatusCode::INTERNAL_SERVER_ERROR)",
    content_attr = "builder(default = mime::IMAGE_BMP.to_string())"
)]
#[derive(Serialize, Deserialize, Debug, TypedBuilder, Default, Eq, PartialEq)]
struct SuccessResp {
    success: bool,
}

#[test]
fn it_works_with_default_meta_attr_value() {
    let actual = SuccessResp::builder().success(false).build();
    let expected = SuccessResp {
        success: false,
        content_type: mime::IMAGE_BMP.to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    };

    assert_eq!(actual, expected);
}
