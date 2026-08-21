//! Validation extractors that combine deserialization with validation.

use actix_web::{dev::Payload, web::Form, web::Json, web::Query, Error, FromRequest, HttpRequest};
use serde::de::DeserializeOwned;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

/// Trait for types that can validate themselves.
pub trait Validator {
    /// Validate the data, returning an error message if invalid.
    fn validate(&self) -> Result<(), String>;
}

/// Result of a validation.
pub type ValidationResult = Result<(), String>;

// ---- ValidatedJson -------------------------------------------------------

/// JSON extractor that also validates the deserialized type.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser { name: String, email: String }
///
/// impl Validator for CreateUser {
///     fn validate(&self) -> Result<(), String> {
///         if self.name.is_empty() {
///             return Err("name cannot be empty".into());
///         }
///         if !self.email.contains('@') {
///             return Err("invalid email".into());
///         }
///         Ok(())
///     }
/// }
///
/// async fn handler(ValidatedJson(user): ValidatedJson<CreateUser>) -> impl Responder {
///     HttpResponse::Ok().json(serde_json::json!({"name": user.name}))
/// }
/// ```
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    /// Create a new ValidatedJson instance.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// Unwrap into the inner type.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ValidatedJson<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for ValidatedJson<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Validator + 'static> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned,
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        let mut payload = std::mem::replace(payload, Payload::None);
        Box::pin(async move {
            let json: Json<T> = Json::from_request(&req, &mut payload).await?;
            let inner = json.into_inner();
            inner
                .validate()
                .map_err(actix_web::error::ErrorBadRequest)?;
            Ok(ValidatedJson(inner))
        })
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ValidatedJson<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValidatedJson").field(&self.0).finish()
    }
}

// ---- ValidatedForm -------------------------------------------------------

/// Form extractor that also validates the deserialized type.
pub struct ValidatedForm<T>(pub T);

impl<T> ValidatedForm<T> {
    /// Create a new ValidatedForm instance.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// Unwrap into the inner type.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ValidatedForm<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Validator + DeserializeOwned + 'static> FromRequest for ValidatedForm<T> {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        let mut payload = std::mem::replace(payload, Payload::None);
        Box::pin(async move {
            let form: Form<T> = Form::from_request(&req, &mut payload).await?;
            let inner = form.into_inner();
            inner
                .validate()
                .map_err(actix_web::error::ErrorBadRequest)?;
            Ok(ValidatedForm(inner))
        })
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ValidatedForm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValidatedForm").field(&self.0).finish()
    }
}

// ---- ValidatedQuery ------------------------------------------------------

/// Query extractor that also validates the deserialized type.
pub struct ValidatedQuery<T>(pub T);

impl<T> ValidatedQuery<T> {
    /// Create a new ValidatedQuery instance.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// Unwrap into the inner type.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for ValidatedQuery<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Validator + DeserializeOwned + 'static> FromRequest for ValidatedQuery<T> {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let query: Query<T> = Query::from_request(&req, &mut Payload::None).await?;
            let inner = query.into_inner();
            inner
                .validate()
                .map_err(actix_web::error::ErrorBadRequest)?;
            Ok(ValidatedQuery(inner))
        })
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ValidatedQuery<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValidatedQuery").field(&self.0).finish()
    }
}
