use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{error, HttpRequest, Responder};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use validator::{ValidationError, ValidationErrors};

pub type Response = Result<HttpResponse, HttpError>;

pub enum HttpResponsePayload {
    Json(serde_json::Value),
    Empty,
}

/// A HTTP response
/// Original http response from actix_web can not be shared between threads
/// and cant be used inside async blocks
/// This struct is a wrapper around actix_web::HttpResponse that can be shared between threads.
pub struct HttpResponse {
    pub status: StatusCode,
    pub payload: HttpResponsePayload,
    pub headers: Vec<(String, String)>,
}

pub struct HttpResponseBuilder {
    status: StatusCode,
    headers: Vec<(String, String)>,
}

impl HttpResponseBuilder {
    pub fn new(status: StatusCode) -> Self {
        HttpResponseBuilder {
            status,
            headers: vec![],
        }
    }

    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    pub fn finish(self) -> HttpResponse {
        HttpResponse {
            status: self.status.clone(),
            payload: HttpResponsePayload::Empty,
            headers: self.headers.clone(),
        }
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

impl Responder for HttpResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut http_response_builder = actix_web::HttpResponse::build(self.status);

        for (key, value) in self.headers {
            http_response_builder.insert_header((key, value));
        }

        match self.payload {
            HttpResponsePayload::Json(value) => http_response_builder
                .content_type("application/json")
                .json(value),
            HttpResponsePayload::Empty => http_response_builder.finish(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationErrorResponse {
    pub validation_errors: Vec<ValidationError>,
}

impl ValidationErrorResponse {
    pub fn from(validation_errors: ValidationErrors) -> ValidationErrorResponse {
        let validation_errors = validation_errors
            .field_errors()
            .into_values()
            .flat_map(|v| v.clone())
            .collect();

        ValidationErrorResponse { validation_errors }
    }
}

impl Display for ValidationErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.validation_errors)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpErrorDetailsResponse {
    pub message: String,
}

#[derive(Debug, Display)]
pub enum HttpError {
    DatabaseError(sqlx::Error),
    ValidationError(ValidationErrorResponse),
    WithDetails(HttpErrorDetails),
}

impl Error for HttpError {}

impl error::ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            HttpError::ValidationError(_) => StatusCode::BAD_REQUEST,
            HttpError::WithDetails(details) => details.status_code,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let mut http_response_builder = actix_web::HttpResponse::build(self.status_code());
        http_response_builder.insert_header(ContentType::json());

        match self {
            HttpError::DatabaseError(er) => http_response_builder.json(HttpErrorDetailsResponse {
                message: er.to_string(),
            }),
            HttpError::ValidationError(er) => http_response_builder.json(er),
            HttpError::WithDetails(details) => {
                for (key, value) in details.headers.iter() {
                    http_response_builder.insert_header((key.clone(), value.clone()));
                }
                http_response_builder.json(HttpErrorDetailsResponse {
                    message: details.message.clone(),
                })
            }
        }
    }
}

impl From<ValidationErrors> for HttpError {
    fn from(validation_errors: ValidationErrors) -> Self {
        HttpError::ValidationError(ValidationErrorResponse::from(validation_errors))
    }
}

impl From<sqlx::Error> for HttpError {
    fn from(e: sqlx::Error) -> Self {
        HttpError::DatabaseError(e)
    }
}

#[derive(Debug, Clone)]
pub struct HttpErrorDetails {
    pub message: String,
    pub status_code: StatusCode,
    pub headers: Vec<(String, String)>,
}

impl Display for HttpErrorDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "{:?}: {:?} ({:?})",
            self.status_code, self.message, headers
        )
    }
}

macro_rules! http_response_builder {
    ($name:ident,$status:expr) => {
        impl HttpResponse {
            #[allow(non_snake_case, missing_docs)]
            pub fn $name() -> HttpResponseBuilder {
                HttpResponseBuilder::new($status)
            }
        }
    };
}

http_response_builder!(BadRequest, StatusCode::BAD_REQUEST);
http_response_builder!(Ok, StatusCode::OK);
http_response_builder!(Created, StatusCode::CREATED);
http_response_builder!(NotFound, StatusCode::NOT_FOUND);

macro_rules! http_error {
    ($name:ident,$status_code:expr) => {
        #[allow(missing_docs, unused)]
        pub fn $name<T>(message: impl Into<String>) -> Result<T, HttpError> {
            Err(HttpError::WithDetails(HttpErrorDetails {
                message: message.into(),
                status_code: $status_code,
                headers: vec![],
            }))
        }
    };
}

http_error!(conflict, StatusCode::CONFLICT);

http_error!(unauthorized, StatusCode::UNAUTHORIZED);

http_error!(bad_request, StatusCode::BAD_REQUEST);

http_error!(not_found, StatusCode::NOT_FOUND);

http_error!(internal_server_error, StatusCode::INTERNAL_SERVER_ERROR);

macro_rules! http_response {
    ($name:ident,$status:ident) => {
        #[allow(non_snake_case, missing_docs)]
        pub fn $name(value: impl Serialize + 'static) -> Response {
            Ok(HttpResponse::$status().json(value))
        }
    };
}

http_response!(ok, Ok);
