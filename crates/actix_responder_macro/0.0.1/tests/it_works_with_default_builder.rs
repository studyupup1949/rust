extern crate actix_responder_macro;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use typed_builder::TypedBuilder;

#[actix_responder(meta_attr = "builder(default)")]
#[derive(Serialize, Deserialize, Debug, TypedBuilder, Default)]
struct SuccessResp {
    success: bool,
}

#[test]
fn it_works_with_default_builder() {
    let a = SuccessResp::builder().success(true).build();
    assert_eq!(
        json!(a),
        json!(SuccessResp {
            success: true,
            successresp_metadata: SuccessRespMetadata {
                status_code: None,
                content_type: None,
            },
        })
    )
}
