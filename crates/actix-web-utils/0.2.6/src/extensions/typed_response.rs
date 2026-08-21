use actix_web::{HttpResponse, http::StatusCode, web, HttpRequest, HttpResponseBuilder, body::BoxBody, Responder};
use serde::Serialize;

use crate::dtos::message::MessageResource;

    /// Defines a type for actix web routes. As the current implementation of HttpResponse doesn't let you manually specify a type.
    /// ```
    /// 
    /// use actix_web::{web::{Path}, http::StatusCode};
    /// use actix_web_utils::extensions::typed_response::TypedHttpResponse;
    /// use actix_web_utils::dtos::message::MessageResource;
    /// 
    /// //Sample route
    /// pub async fn testroute(number: Path<i32>) -> TypedHttpResponse<String> {
    ///     if(*number > 0){
    ///         return TypedHttpResponse::return_standard_response(StatusCode::OK, String::from("This is my test response!")); 
    ///     }
    ///     TypedHttpResponse::return_empty_response(StatusCode::BAD_REQUEST)
    /// }
    /// 
    /// ```
pub struct TypedHttpResponse<B: Serialize = String> {
    pub response: HttpResponse<Option<web::Json<Result<B, Vec<MessageResource>>>>>,
}
impl<B: Serialize> TypedHttpResponse<B> {
    /// Returns a response with the json struct you define inside + Status code
    /// ```
    /// use actix_web_utils::extensions::typed_response::TypedHttpResponse;
    /// 
    /// TypedHttpResponse::return_standard_response(200, String::from("Anything in here")); //Instead of string you can put anything here as long as it is serializable.
    /// 
    /// ```
    pub fn return_standard_response(status_code: u16, body: B) -> TypedHttpResponse<B>{
        TypedHttpResponse { 
            response: HttpResponse::with_body(StatusCode::from_u16(u16::from(status_code)).unwrap(), Some(web::Json(Ok(body))))
        }
    }
    ///  Returns a response with the json error list inside + Status code
    pub fn return_standard_error_list(status_code: u16, body: Vec<MessageResource>) -> TypedHttpResponse<B>{
        TypedHttpResponse {
            response: HttpResponse::with_body(StatusCode::from_u16(u16::from(status_code)).unwrap(), Some(web::Json(Err(body))))
        }
    }
    /// This is to return a MessageResource wrapped inside an HttpResponse
    pub fn return_standard_error(status_code: u16, body: MessageResource) -> TypedHttpResponse<B>{
        TypedHttpResponse { 
            response: HttpResponse::with_body(StatusCode::from_u16(u16::from(status_code)).unwrap(), Some(web::Json(Err(vec![body]))))
        }
    }
    /// Returns an empty http response with a status code
    /// This is a bad practice, but I still left it here
    /// as it is useful for debugging & specific cases
    pub fn return_empty_response(status_code: u16) -> TypedHttpResponse<B>{
        TypedHttpResponse { 
            response: HttpResponse::with_body(StatusCode::from_u16(u16::from(status_code)).unwrap(), None)
        }
    }
}

impl<T: Serialize> Responder for TypedHttpResponse<T>
{
    type Body = BoxBody;
    #[inline]
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut builder = HttpResponseBuilder::new(self.response.status());
        match self.response.body() {
            Some(body) => {match body.0.as_ref() {
                Ok(value) => builder.json(value),
                Err(errors) => builder.json(errors),
            }},
            None => {builder.finish()}
        }
    }
}

