//! Ergonomic extractors that eliminate boilerplate.
//!
//! These extractors wrap the standard Actix extractors and automatically
//! unwrap the inner value, so you never need to call `.into_inner()`.
//!
//! # Example
//!
//! ```no_run
//! use actix_tower::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CreateUser { name: String, email: String }
//!
//! async fn handler(AutoJson(user): AutoJson<CreateUser>) -> impl Responder {
//!     // user is CreateUser, not Json<CreateUser>
//!     HttpResponse::Ok().json(serde_json::json!({"id": 1, "name": user.name}))
//! }
//! ```

pub mod data;
pub mod form;
pub mod json;
pub mod multipart;
pub mod path;
pub mod query;
pub mod state;
pub mod validation;

pub use data::AutoData;
pub use form::AutoForm;
pub use json::AutoJson;
pub use multipart::AutoMultipart;
pub use path::AutoPath;
pub use query::AutoQuery;
pub use state::AutoState;

pub use validation::{ValidatedForm, ValidatedJson, ValidatedQuery, ValidationResult};
