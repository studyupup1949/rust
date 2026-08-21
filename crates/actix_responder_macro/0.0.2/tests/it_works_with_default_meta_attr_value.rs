extern crate actix_responder_macro;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[actix_responder(meta_attr = r#"builder(
        default = SuccessRespMetadata{
                status_code: Some(StatusCode::INTERNAL_SERVER_ERROR),
                content_type: Some("image/bmp".to_string())
            }
        )"#)]
#[derive(Serialize, Deserialize, Debug, TypedBuilder, Default)]
struct SuccessResp {
    success: bool,
}

#[test]
fn it_works_with_default_meta_attr_value() {
    let a = SuccessResp::builder().success(false).build();
    assert!(!a.success);
    assert_eq!(
        a.metadata.status_code,
        Some(StatusCode::INTERNAL_SERVER_ERROR)
    );
    assert_eq!(a.metadata.content_type, Some("image/bmp".to_string()));

    let a = SuccessResp::builder()
        .success(true)
        .metadata(SuccessRespMetadata {
            status_code: Some(StatusCode::OK),
            content_type: Some("application/json".to_string()),
        })
        .build();

    assert!(a.success);
    assert_eq!(a.metadata.status_code, Some(StatusCode::OK));
    assert_eq!(
        a.metadata.content_type,
        Some("application/json".to_string())
    );
}
