use actix_web::{HttpResponse, ResponseError};
use std::collections::HashMap;
use std::fmt;

/// Errors that can occur during admin operations.
#[derive(Debug)]
pub enum AdminError {
    NotFound,
    ValidationError(HashMap<String, String>),
    DatabaseError(String),
    Unauthorized,
}

impl fmt::Display for AdminError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdminError::NotFound => write!(f, "Resource not found"),
            AdminError::ValidationError(_) => write!(f, "Validation failed"),
            AdminError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AdminError::Unauthorized => write!(f, "Unauthorized access"),
        }
    }
}

impl ResponseError for AdminError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AdminError::NotFound => HttpResponse::NotFound().finish(),
            AdminError::ValidationError(_) => HttpResponse::BadRequest().finish(),
            AdminError::DatabaseError(_) => HttpResponse::InternalServerError().finish(),
            AdminError::Unauthorized => HttpResponse::Unauthorized().finish(),
        }
    }
}
