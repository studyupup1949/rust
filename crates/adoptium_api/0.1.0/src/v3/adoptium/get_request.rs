//! Provides the `GetRequest` trait for making simple GET requests to the Adoptium API.
//!
//! This module defines a common interface for request types that can be turned into URLs and fetched asynchronously,
//! returning a deserialized response.

use crate::v3::adoptium::{ApiBase, Error, TryAsUrl};


/// A trait for types that represent a GET request to the Adoptium API.
///
/// Types implementing this trait must be convertible into a URL using [`TryAsUrl`],
/// and specify a response type that can be deserialized from JSON.
pub trait GetRequest: TryAsUrl {
    /// The type returned after the request is completed and the JSON body is parsed.
    type Response: serde::de::DeserializeOwned;

    /// Performs the GET request using the provided [`ApiBase`].
    ///
    /// # Arguments
    /// * `base` - The API base URL (e.g. production or staging endpoint).
    ///
    /// # Returns
    /// An asynchronous future resolving to:
    ///   - `Ok(Self::Response)` on success
    ///   - `Err(Error)` if the URL conversion, network request, or JSON parsing fails
    fn get(&self, base: &ApiBase) -> impl Future<Output = Result<Self::Response, Error>> { async {
        let url = self.try_as_url(base)?;

        reqwest::get(url).await?
            .json::<Self::Response>().await
            .map_err(|e| Error::Serde(e.to_string()))
    } }
}
