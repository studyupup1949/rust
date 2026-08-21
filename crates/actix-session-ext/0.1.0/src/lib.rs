//! # actix-session-ext
//!
//! The `actix-session-ext` crate provides a safer `actix_session::Session` interface thanks to typed key.
//!
//! ## Examples
//! ```rust,no_run
//! use actix_web::{Error, Responder, HttpResponse};
//! use actix_session::Session;
//! use actix_session_ext::{SessionKey, SessionExt};
//!
//! // create an actix application and attach the session middleware to it
//!
//! const USER_KEY: SessionKey<String> = SessionKey::new("user");
//! const TIMESTAMP_KEY: SessionKey<u64> = SessionKey::new("timestamp");
//!
//! #[actix_web::post("/login")]
//! async fn login(session: Session) -> Result<String, Error> {
//!     session.insert_by_key(USER_KEY, "Dupont".to_owned())?;
//!     session.insert_by_key(TIMESTAMP_KEY, 1234567890)?;
//!
//!    Ok("logged in".to_owned())
//! }
//!
//! #[actix_web::get("/logged_at")]
//! async fn logged_at(session: Session) -> Result<String, Error> {
//!    let timestamp = session.get_by_key(TIMESTAMP_KEY)?.unwrap_or_default();
//!
//!    Ok(format!("logged at {}", timestamp))
//! }
//! ```

#![deny(clippy::pedantic)]

use std::marker::PhantomData;

use actix_session::{Session, SessionGetError, SessionInsertError};
use serde::{de::DeserializeOwned, Serialize};

/// `SessionKey<T>`, a struct binding a key to its respective type.
///
/// This type is useful to avoid common pitfalls when using raw key:
/// - Mistyped key on both insertion/retrieval
/// - Wrong type casting on retrieval
pub struct SessionKey<T> {
  value: &'static str,
  _marker: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> SessionKey<T> {
  /// Constructs a typed session key.
  #[must_use]
  pub const fn new(value: &'static str) -> Self {
    Self {
      value,
      _marker: PhantomData,
    }
  }

  /// Returns the raw key as a string.
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    self.value
  }
}

/// An extension trait for `actix_session::session` that provides a safer alternative to
/// `session::get`, `session::insert`, `session::remove` methods.
/// This trait is implemented for `actix_session::session` and provides methods that take
/// `SessionKey<T>` instead of raw keys.
///
pub trait SessionExt {
  /// Get a `value` from the session by `key`.
  ///
  /// # Errors
  /// It returns an error if the value is not found or if the value is not of the expected type.
  fn get_by_key<T: DeserializeOwned>(
    &self,
    key: SessionKey<T>,
  ) -> Result<Option<T>, SessionGetError>;

  /// Insert a `value` into the session by `key`.
  ///
  /// # Errors
  /// It returns an error if the value cannot be serialized.
  fn insert_by_key<T: Serialize>(
    &self,
    key: SessionKey<T>,
    value: T,
  ) -> Result<(), SessionInsertError>;

  /// Remove a `value` from the session by `key`.
  ///
  /// # Errors
  /// It returns an error if the value cannot be deserialized.
  fn remove_by_key<T: DeserializeOwned>(&self, key: SessionKey<T>) -> Result<Option<T>, String>;
}

impl SessionExt for Session {
  fn get_by_key<T: DeserializeOwned>(
    &self,
    key: SessionKey<T>,
  ) -> Result<Option<T>, SessionGetError> {
    self.get::<T>(key.value)
  }

  fn insert_by_key<T: Serialize>(
    &self,
    key: SessionKey<T>,
    value: T,
  ) -> Result<(), SessionInsertError> {
    self.insert(key.value, value)
  }

  fn remove_by_key<T: DeserializeOwned>(&self, key: SessionKey<T>) -> Result<Option<T>, String> {
    self.remove_as::<T>(key.value).transpose()
  }
}
