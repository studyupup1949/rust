pub use actix_response_macro::ResponseError;
use actix_web::{HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ApiResponse<T> {
    #[serde(rename = "success")]
    Success(T),
    #[serde(rename = "error")]
    Error(String),
}

#[derive(Serialize)]
pub struct ApiSuccessModel<T> {
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: T,
}

#[derive(Serialize)]
pub struct ApiErrorModel {
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: String,
}

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = actix_web::body::BoxBody;
    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok().json(self)
    }
}
