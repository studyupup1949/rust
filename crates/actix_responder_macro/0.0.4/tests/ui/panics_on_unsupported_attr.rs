extern crate actix_responder_macro;
extern crate typed_builder;

use actix_responder_macro::actix_responder;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[actix_responder(imaginary_attr = "builder(default)")]
#[derive(Serialize, Deserialize, Debug, TypedBuilder, Default)]
struct SuccessResp {
    success: bool,
}

fn main() {}
