//! Types to manage errors in Actix apps.

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use log::error;
use serde::{Deserialize, Serialize};

#[cfg(feature = "sqlx")]
use sqlx::Error as SqlxError;

use std::collections::HashMap;
use validator::{ValidationError, ValidationErrors};

/// Use to serialize a simple error with a static message.
#[derive(Debug, Serialize)]
pub struct InternalErrorPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<&'static str>,
    pub error: &'static str,
}

impl InternalErrorPayload {
    pub fn init(error: &'static str) -> Self {
        Self {
            code: None,
            error,
        }
    }
}

/// Use to serialize a validation
/// with a string error and/or field validation errors.
///
/// An error serialized as JSON looks like:
///
/// ```json
/// {
///   "error": "Validation error",
///   "field_errors": {
///     "name": [
///       {
///         "code": "length",
///         "message": null,
///         "params": { "min": 3, "value": "Sr" }
///       }
///     ]
///   }
/// }
/// ```
#[derive(Debug, Deserialize, Serialize)]
pub struct ValidationErrorPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<HashMap<String, Vec<ValidationError>>>,
}

impl ValidationErrorPayload {
    pub fn new(detail: String) -> Self {
        ValidationErrorPayload {
            code: None,
            error: detail,
            field_errors: None,
        }
    }

    pub fn with_code(code_error: String, detail: String) -> Self {
        ValidationErrorPayload {
            code: Some(code_error),
            error: detail,
            field_errors: None,
        }
    }
}

impl From<&ValidationErrors> for ValidationErrorPayload {
    fn from(error: &ValidationErrors) -> Self {
        let mut errors: HashMap<String, Vec<ValidationError>> = HashMap::new();
        errors.extend(
            error
                .field_errors()
                .iter()
                .map(|(k, v)| (String::from(k.clone()), (*v).clone())),
        );
        ValidationErrorPayload {
            code: Some("validation_error".to_owned()),
            error: if errors.len() > 1 { "Validations error".to_owned() } else { "Validation error".to_owned() },
            field_errors: Some(errors),
        }
    }
}

/// Main enum that implements the actix [ResponseError](https://actix.rs/docs/errors/)
/// trait to be used as wrapper for different errors
/// in endpoint handlers.
///
/// See [`HttpResult`].
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    /// Used to trigger any validation where the error
    /// message doesn't need to be generated (string reference).
    ///
    /// These errors are processed as `HTTP 400 Bad Request`.
    ///
    /// # Example
    /// ```ignore, no_run
    /// use actix_contrib_rest::result::AppError;
    /// // ...
    /// return Err(AppError::StaticValidation(
    ///     "User already exists."
    /// ));
    /// ```
    #[error("{0}")]
    StaticValidation(&'static str),

    /// Used to trigger any validation where you need to
    /// build the string with the error details.
    ///
    /// These errors are processed as `HTTP 400 Bad Request`.
    ///
    /// # Example
    /// ```ignore, no_run
    /// use actix_contrib_rest::result::AppError;
    /// // ...
    /// return Err(AppError::Validation(
    ///     Some("insufficient_funds"),
    ///     format!("Customer with account {} doesn't have enough funds.", account.id)
    /// ));
    /// ```
    #[error("{1}")]
    Validation(Option<&'static str>, String),

    #[cfg(feature = "sqlx")]
    /// Encapsulates a `SqlxError` error (database errors), like
    /// the DB is not accessible, time outs, and so on.
    ///
    /// These errors are processed as `HTTP 500 Internal Server Error`.
    /// # Example
    /// ```ignore, no_run
    /// use actix_contrib_rest::result::AppError;
    /// // ...
    /// let customer = sqlx::query_as!(
    ///     Tenant,
    ///     "SELECT id, name, created_at FROM customers WHERE id = $1", id
    /// )
    /// .fetch_optional(&mut **tx)
    /// .await
    /// .map_err(AppError::DB)?;    // If there is a DB error, it's mapped here
    /// ```
    #[error(transparent)]
    DB(#[from] SqlxError),

    /// Used when a resource requested cannot be found,
    /// or was deleted.
    ///
    /// These errors are processed as `HTTP 404 Not Found`.
    ///
    /// # Example
    /// ```ignore, no_run
    /// use actix_contrib_rest::result::AppError;
    /// // ...
    /// return Err(AppError::ResourceNotFound {
    ///     resource: "order",
    ///     attribute: "id",
    ///     value: order.id.to_string()
    /// });
    /// ```
    ///
    /// In the example above, the error message will be:
    /// *order with id equals to "123432" not found or was removed*.
    #[error("{resource} with {attribute} equals to \"{value}\" not found or was removed")]
    ResourceNotFound {
        resource: &'static str,
        attribute: &'static str,
        value: String,
    },

    /// Used when a resource already exists in the system.
    ///
    /// These errors are processed as `HTTP 400 Bad Request`.
    ///
    /// # Example
    /// ```ignore, no_run
    /// use actix_contrib_rest::result::AppError;
    /// // ...
    /// return Err(AppError::ResourceAlreadyExists {
    ///     resource: "invoice",
    ///     attribute: "number",
    ///     value: invoice.id.to_string()
    /// });
    /// ```
    ///
    /// In the example above, the error message will be:
    /// *invoice with number "123432" already exists*.
    #[error("{resource} with {attribute} \"{value}\" already exists")]
    ResourceAlreadyExists {
        resource: &'static str,
        attribute: &'static str,
        value: String,
    },

    /// Lacks valid authentication credentials for the requested resource.
    ///
    /// These errors are processed as `HTTP 401 Unauthorized`.
    ///
    /// # Example
    /// ```ignore, no_run
    /// use actix_contrib_rest::result::AppError;
    /// // ...
    /// return Err(AppError::Unauthorized(
    ///     "Unauthorized access to ..."
    /// ));
    /// ```
    #[error("{0}")]
    Unauthorized(&'static str),

    /// Use to indicates that system understands the request but refuses
    /// to authorize it.
    ///
    /// These errors are processed as `HTTP 403 Forbidden`.
    ///
    /// # Example
    /// ```ignore, no_run
    /// use actix_contrib_rest::result::AppError;
    /// // ...
    /// return Err(AppError::Forbidden(
    ///     "Cannot access to ..."
    /// ));
    /// ```
    #[error("{0}")]
    Forbidden(&'static str),

    /// Any other error that needs to be wrapped inside an AppError.
    ///
    /// These errors are processed as `HTTP 500 Internal Server Error`.
    ///
    /// # Example
    /// Having an Error `e`, can be used as follow:
    /// ```ignore, no_run
    /// use actix_contrib_rest::result::AppError;
    /// // ...
    /// return Err(AppError::Unexpected(e.into()));
    /// ```
    /// Or something like:
    /// ```ignore, no_run
    /// some_operation().map_err(|e| AppError::Unexpected(e.into()))?;
    /// ```
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::StaticValidation(_) | Self::Validation(_, _) => StatusCode::BAD_REQUEST,
            Self::ResourceAlreadyExists { resource: _, attribute: _, value: _ } => StatusCode::BAD_REQUEST,
            Self::ResourceNotFound { resource: _, attribute: _, value: _ } => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
            #[cfg(feature = "sqlx")]
            Self::DB(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        match self {
            Self::Validation(code, error) => {
                match code {
                    None => HttpResponse::build(status_code)
                        .json(ValidationErrorPayload::new(error.to_owned())),
                    Some(c) =>
                        HttpResponse::build(status_code)
                            .json(ValidationErrorPayload::with_code(c.to_string(), error.to_owned())),
                }
            }
            Self::StaticValidation(error)
                | Self::Unauthorized(error) | Self::Forbidden(error) => {
                HttpResponse::build(status_code)
                    .json(InternalErrorPayload::init(error))
            }
            Self::ResourceNotFound { resource: _, attribute: _, value: _ } => {
                HttpResponse::build(status_code)
                    .json(ValidationErrorPayload::with_code(
                        "not_found".to_string(),
                        self.to_string(),
                    ))
            }
            Self::ResourceAlreadyExists { resource: _, attribute: _, value: _ } => {
                HttpResponse::build(status_code)
                    .json(ValidationErrorPayload::with_code(
                        "already_exists".to_string(),
                        self.to_string(),
                    ))
            }
            _ => {
                HttpResponse::build(status_code)
                    .json(InternalErrorPayload::init(
                        status_code.canonical_reason().unwrap_or("Unknown error")
                    ))
            }
        }
    }
}

/// Type to use as result for a request handlers in order
/// to allow [`AppError`] to handle properly response
/// errors.
/// See [`HttpResult`].
pub type Result<T> = core::result::Result<T, AppError>;

/// Type to use as return for a request handlers in order
/// to allow [`AppError`] to handle properly response
/// errors.
/// # Example
/// ```ignore, no_run
/// use actix_contrib_rest::result::HttpResult;
/// use actix_web::{patch, web, HttpResponse};
/// use actix_web::web::{Data, Path};
/// use actix_web_validator::Json;
/// use serde::Deserialize;
/// use validator::Validate;
///
/// #[derive(Deserialize, Validate)]
/// struct SalePayload {
///     #[validate(length(min = 3, max = 80))]
///     pub name: String,
///     pub prod_id: u32,
///     // ...
/// }
///
/// #[patch("{id}")]
/// async fn read(
///     app: Data<AppState>,
///     id: Path<String>,
///     sale_form: Json<SalePayload>    // If there is an error handling the JSON serialization of
///                                     // SalePayload (like serde validations), it will
///                                     // managed as an HTTP 400 error with a proper JSON response
/// ) -> HttpResult {                   // `HttpResult` is the return type and the error handler as well
///     let mut tx = app.get_tx().await?;   // Possible `AppError` error
///     let sale_order = Order::save(
///         &mut tx,
///         id.into_inner().as_str(),
///         sale_form.0
///     ).await?;                   // Here an `AppError` can be returned, but `HttpResult` will handle it
///     app.commit_tx(tx).await?;   // Another possible `AppError`
///     match sale_order {
///         Some(sale) => Ok(HttpResponse::Ok().json(sale)),
///         None => Ok(HttpResponse::NotFound().finish()),
///     }
/// }
/// ```
pub type HttpResult = Result<HttpResponse>;


/// Use to serialize the number of elements
/// where deleted after a request was made.
#[derive(Debug, Deserialize, Serialize)]
pub struct DeletedCount {
    pub deleted: u64,
}
