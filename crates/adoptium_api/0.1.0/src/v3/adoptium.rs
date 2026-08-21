//! Adoptium API v3 client utilities.
//!
//! This module groups together all core components for working with the Adoptium API v3.
//! It provides:
//!   - [`Error`] – unified error type for request and parsing failures.
//!   - [`ApiBase`] – wrapper for selecting production or staging API endpoints.
//!   - [`TryAsUrl`] – trait for converting endpoint types into full URLs.
//!   - [`GetRequest`] – trait for endpoints that can perform GET requests.
//!   - [`Adoptium`] – facade for combining a base URL and endpoint into a request.

mod error;
pub use error::Error;

mod api_base;
pub use api_base::ApiBase;

mod try_as_url;
pub use try_as_url::TryAsUrl;

mod get_request;
pub use get_request::GetRequest;

mod client_facade;
pub use client_facade::Adoptium;
