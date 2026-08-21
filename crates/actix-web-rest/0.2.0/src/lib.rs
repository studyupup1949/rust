pub use actix_web;
pub use actix_web_rest_macros;
pub use actix_web_rest_macros::*;
pub use http;
pub use serde;
pub use strum;

pub trait RestError {
    fn status_code(&self) -> http::StatusCode;
}
