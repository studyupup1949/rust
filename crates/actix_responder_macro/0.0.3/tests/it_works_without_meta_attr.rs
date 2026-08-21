extern crate actix_responder_macro;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[actix_responder]
#[derive(Serialize, Deserialize, Debug, Default)]
struct SuccessResp {
    success: bool,
}

#[test]
fn it_works_without_meta_attr() {
    let a = SuccessResp::default();
    assert_eq!(
        json!(a),
        json!(SuccessResp {
            success: false,
            metadata: SuccessRespMetadata {
                status_code: None,
                content_type: None,
            },
        })
    )
}
