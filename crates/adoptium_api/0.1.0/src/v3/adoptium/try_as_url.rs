//! Provides the `TryAsUrl` trait for converting request types into URLs.
//!
//! This module defines a simple interface for building URLs from request objects using a given [`ApiBase`].

use crate::v3::adoptium::{ApiBase, Error};


/// A trait for converting a request type into a fully qualified [`url::Url`].
///
/// Types implementing this trait define how they can be transformed into a URL using the provided API base.
pub trait TryAsUrl {
    /// Attempts to build a URL from the request type and the given [`ApiBase`].
    ///
    /// # Arguments
    /// * `base` - The API base URL (e.g. production or staging endpoint).
    ///
    /// # Returns
    ///   - `Ok(url::Url)` if the URL was successfully built.
    ///   - `Err(Error)` if URL construction failed.
    fn try_as_url(&self, base: &ApiBase) -> Result<url::Url, Error>;
}
