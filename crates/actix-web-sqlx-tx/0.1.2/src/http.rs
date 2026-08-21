use std::error::Error;

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{error, HttpRequest, Responder};
use derive_more::Display;
use serde::Serialize;

pub type Response = Result<HttpResponse, HttpError>;

/// A HTTP response
/// Original http response from actix_web can not be shared between threads
/// and cant be used inside async blocks
/// This struct is a wrapper around actix_web::HttpResponse that can be shared between threads.
pub struct HttpResponse {
    pub status: StatusCode,
    pub payload: HttpResponsePayload,
    pub headers: Vec<(String, String)>,
}

macro_rules! static_response {
    ($name:ident,$status:expr) => {
        impl HttpResponse {
            #[allow(non_snake_case, missing_docs)]
            pub fn $name() -> HttpResponseBuilder {
                HttpResponseBuilder::new($status)
            }
        }
    };
}

static_response!(BadRequest, StatusCode::BAD_REQUEST);
static_response!(Ok, StatusCode::OK);
static_response!(Created, StatusCode::CREATED);

pub struct HttpResponseBuilder {
    status: StatusCode,
    headers: Vec<(String, String)>,
}

pub enum HttpResponsePayload {
    Json(serde_json::Value),
}

impl Responder for HttpResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut http_response_builder = actix_web::HttpResponse::build(self.status);

        for (key, value) in self.headers {
            http_response_builder.insert_header((key, value));
        }
        match self.payload {
            HttpResponsePayload::Json(body) => http_response_builder
                .content_type("application/json")
                .json(body),
        }
    }
}

impl HttpResponseBuilder {
    pub fn new(status: StatusCode) -> Self {
        HttpResponseBuilder {
            status,
            headers: vec![],
        }
    }

    pub fn add_header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self
    }

    pub fn json<T>(&self, value: T) -> HttpResponse
    where
        T: Serialize + 'static,
    {
        match serde_json::to_value(&value) {
            Ok(body) => HttpResponse {
                status: self.status.clone(),
                payload: HttpResponsePayload::Json(body),
                headers: self.headers.clone(),
            },
            Err(_) => {
                panic!("Failed to serialize response body");
            }
        }
    }
}

#[derive(Debug, Display)]
pub enum HttpError {
    DatabaseError(sqlx::Error),
}

impl From<sqlx::Error> for HttpError {
    fn from(e: sqlx::Error) -> Self {
        HttpError::DatabaseError(e)
    }
}

#[derive(Serialize, Clone)]
pub struct HttpErrorDetails {
    pub message: String,
}

impl Error for HttpError {}

impl error::ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            HttpError::DatabaseError(er) => {
                actix_web::HttpResponse::InternalServerError().json(HttpErrorDetails {
                    message: er.to_string(),
                })
            }
        }
    }
}
