extern crate actix_responder_macro;

use actix_responder_macro::actix_responder;
use serde::{Deserialize, Serialize};

#[actix_responder(meta_attr = "")]
#[derive(Serialize, Deserialize, Debug, Default)]
struct SuccessResp {
    success: bool,
}

fn main() {}
