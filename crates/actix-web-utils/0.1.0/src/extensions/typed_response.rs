use actix_web::{HttpResponse, http::StatusCode, web, HttpRequest, HttpResponseBuilder, body::BoxBody, Responder};
use serde::Serialize;

use crate::dtos::message::MessageResource;


pub struct TypedHttpResponse<B: Serialize> {
    pub response: HttpResponse<Option<web::Json<Result<B, Vec<MessageResource>>>>>,
}
impl<B: Serialize> TypedHttpResponse<B> {
    //  Returns a response with the json struct inside + Status code
    pub fn return_standard_response(status_code: StatusCode, body: B) -> TypedHttpResponse<B>{
        TypedHttpResponse { 
            response: HttpResponse::with_body(StatusCode::from_u16(u16::from(status_code)).unwrap(), Some(web::Json(Ok(body))))
        }
    }
    //  Returns a response with the json error list inside + Status code
    pub fn return_standard_error_list(status_code: StatusCode, body: Vec<MessageResource>) -> TypedHttpResponse<B>{
        TypedHttpResponse {
            response: HttpResponse::with_body(StatusCode::from_u16(u16::from(status_code)).unwrap(), Some(web::Json(Err(body))))
        }
    }
    pub fn return_standard_error(status_code: StatusCode, body: MessageResource) -> TypedHttpResponse<B>{
        TypedHttpResponse { 
            response: HttpResponse::with_body(StatusCode::from_u16(u16::from(status_code)).unwrap(), Some(web::Json(Err(vec![body]))))
        }
    }
    //  Returns an empty response with status code
    pub fn return_empty_response(status_code: StatusCode) -> TypedHttpResponse<B>{
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