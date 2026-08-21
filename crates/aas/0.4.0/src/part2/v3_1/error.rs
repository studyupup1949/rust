use axum::extract::FromRequest;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::Display;
use utoipa::ToSchema;

/// All AAS routes that can return an error in this format.
#[derive(Clone, Deserialize, Serialize, Debug, ToSchema)]
#[serde(untagged)]
pub enum AASError {
    NotFound { messages: Vec<AASMessage> },
    BadRequest { messages: Vec<AASMessage> },
    Unauthorized { messages: Vec<AASMessage> },
    Forbidden { messages: Vec<AASMessage> },
    Internal { messages: Vec<AASMessage> },
}

/// A message containing more information for
/// the requester about a certain happening in the backend
#[derive(Clone, Deserialize, Serialize, Debug, ToSchema)]
pub struct AASMessage {
    #[serde(rename = "messageType")]
    pub message_type: AASErrorMessageType,

    pub code: String,

    /// Identifier to relate several result messages throughout several systems
    #[serde(rename = "correlationId")]
    pub correlation_id: String,

    pub text: String,
    pub timestamp: DateTime<chrono::Utc>,
}

#[derive(Clone, Deserialize, Serialize, Default, Debug, Display, ToSchema)]
pub enum AASErrorMessageType {
    #[default]
    Undefined,
    /// Used to inform the user about a certain fact
    Info,
    /// Used for warnings; warnings may lead to errors in the subsequent execution
    Warning,
    /// Used for handling errors
    Error,
    /// Used in case of an internal and/or unhandled exception
    Exception,
}

// Tell axum how `AppError` should be converted into a response.
impl IntoResponse for AASError {
    fn into_response(self) -> Response {
        let (status, err) = match &self {
            AASError::NotFound { messages: _ } => (StatusCode::NOT_FOUND, self),
            AASError::Internal { messages: _ } => (StatusCode::INTERNAL_SERVER_ERROR, self),
            AASError::BadRequest { .. } => (StatusCode::BAD_REQUEST, self),
            AASError::Unauthorized { .. } => (StatusCode::UNAUTHORIZED, self),
            AASError::Forbidden { .. } => (StatusCode::FORBIDDEN, self),
        };

        let mut response = (status, AASErrorJson(err.clone())).into_response();
        response.extensions_mut().insert(Arc::new(err));
        response
    }
}

// Create our own JSON extractor by wrapping `axum::Json`. This makes it easy to override the
// rejection and provide our own which formats errors to match our application.
//
// `axum::Json` responds with plain text if the input is invalid.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AASError))]
struct AASErrorJson<T>(T);

impl<T> IntoResponse for AASErrorJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}
