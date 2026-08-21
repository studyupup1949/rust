use actix_web::body::{BoxBody, MessageBody};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{HttpRequest, HttpResponse, Responder};

pub struct Redirect<B: MessageBody + 'static> {
    to: String,
    body: B,
}

impl Redirect<actix_web::body::None> {
    pub fn new<S: AsRef<str>>(to: S) -> Self {
        Self {
            to: to.as_ref().to_string(),
            body: actix_web::body::None::new(),
        }
    }
}

impl<B: MessageBody + 'static> Redirect<B> {

    #[allow(unused)]
    pub fn new_with_body<S: AsRef<str>>(to: S, body: B) -> Self {
        Self {
            to: to.as_ref().to_string(),
            body,
        }
    }
}

impl<B: MessageBody + 'static> Responder for Redirect<B> {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut builder = HttpResponse::Found();
        builder.append_header((
            HeaderName::from_static("location"),
            HeaderValue::from_str(&self.to).unwrap(),
        ));

        builder.body(self.body)
    }
}
