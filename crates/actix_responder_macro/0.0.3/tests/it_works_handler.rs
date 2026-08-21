extern crate actix_responder_macro;
extern crate actix_web;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use actix_web::get;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[actix_responder(meta_attr = "builder(default)")]
#[derive(Serialize, Deserialize, Debug, TypedBuilder, Default)]
pub struct SuccessResp {
    success: bool,
}

#[get("/health_check")]
pub async fn health_check() -> SuccessResp {
    SuccessResp::builder().success(true).build()
}
