use std::fmt::Display;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpResponse<T = HttpResponseCode, E = HttpError>
where
  T: Serialize,
  E: Serialize + IntoResponse
{
  successful: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  response: Option<T>,
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<E>,
}

impl<T: Serialize, E: Serialize + IntoResponse> HttpResponse<T, E> {
  pub fn new(
    response: T,
  ) -> Json<Self> {
    Json(HttpResponse { successful: true, response: Some(response), error: None })
  }

  pub fn error(
    error: E
  ) -> Json<Self> {
    Json(HttpResponse { successful: false, response: None, error: Some(error) })
  }

  pub fn raw(
    successful: bool,
    response: Option<T>,
    error: Option<E>
  ) -> Json<Self> {
    Json(HttpResponse { successful, response, error })
  }
}

impl HttpResponse {
  pub fn code<M: Into<String>>(
    message: M
  ) -> Json<HttpResponse<HttpResponseCode, HttpError>> {
    Json(HttpResponse { successful: true, response: Some(HttpResponseCode::new(message)), error: None })
  }
}

// #[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
// #[derive(Debug, Deserialize, Serialize)]
// pub struct HttpError(pub anyhow::Error, pub Option<StatusCode>);

pub type RawHttpResult<T> = Result<T, HttpError>;
// wrapping the result into json, 'cuz we are doing rest api
pub type HttpResult<T = HttpResponseCode> = RawHttpResult<Json<HttpResponse<T, HttpError>>>;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpError {
  code: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  details: Option<String>,
  #[serde(skip)]
  status: StatusCode,
}

impl HttpError {
  pub fn new<T: Into<String>, D: Into<String>>(
    code: T,
    details: D,
    status: Option<StatusCode>
  ) -> Self {
    HttpError {
      code: code.into(),
      details: Some(details.into()),
      status: status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }

  pub fn without_details<T: Into<String>  >(
    code: T,
    status: Option<StatusCode>
  ) -> Self {
    HttpError {
      code: code.into(),
      details: None,
      status: status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }

  pub fn internal<T: Into<String>>(
    details: Option<T>,
  ) -> Self {
    HttpError {
      code: "INTERNAL_SERVER_ERROR".to_string(),
      details: details.map(|d| d.into()),
      status: StatusCode::INTERNAL_SERVER_ERROR
    }
  }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponseCode {
  pub code: String
}

impl HttpResponseCode {
  pub fn new<T: Into<String>>(code: T) -> Self {
    HttpResponseCode {
      code: code.into()
    }
  }
}

impl<E> From<E> for HttpError
where
  E: Into<anyhow::Error> + 'static
{
  fn from(err: E) -> Self {
    HttpError::new(err.into().to_string(), "Internal error", None)
  }
}

impl IntoResponse for HttpError {
  fn into_response(self) -> axum::response::Response {
    (
      self.status,
      Json(HttpResponse::<()> {
        successful: false,
        response: None,
        error: Some(self)
      })
    ).into_response()
  }
}

// pub trait IntoHttpError {
//   fn into_internal<'a, T: Deserialize<'a> + Serialize>() -> Result<T, HttpError>;
// }

pub trait CastErrorIntoResponse<T> {
  fn err_into_response(self) -> Result<T, HttpError>;
}

impl<T, E: Display> CastErrorIntoResponse<T> for Result<T, E> {
  fn err_into_response(self) -> Result<T, HttpError> {
    match self {
      Ok(c) => Ok(c),
      Err(err) => Err(HttpError::internal(Some(err.to_string())))
    }
  }
}