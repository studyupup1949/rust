//! High-level facade for constructing and executing Adoptium API requests.
//!
//! The [`Adoptium`] struct in this module wraps the low-level request-building logic,
//! combining an [`ApiBase`] with a specific endpoint.
//! This provides a simple interface for building URLs or performing HTTP GET calls.

use crate::v3::adoptium::{ApiBase, Error, GetRequest, TryAsUrl};


/// A facade for building and sending Adoptium API requests.
///
/// [`Adoptium`] pairs an [`ApiBase`] (production or staging) with an endpoint type that implements [`TryAsUrl`].
/// If the endpoint also implements [`GetRequest`], the facade can execute the request and return the parsed response.
pub struct Adoptium<T: TryAsUrl> {
    base: ApiBase,
    endpoint: T,
}

impl<T: TryAsUrl> Adoptium<T> {
    /// Creates a new `Adoptium` client with a specified API base.
    ///
    /// # Arguments
    /// * `base` - The API base URL (production or staging).
    /// * `endpoint` - The endpoint definition to use.
    pub fn new(base: ApiBase, endpoint: T) -> Self {
        Self {
            base,
            endpoint,
        }
    }

    /// Creates a new client configured to use the production API base.
    pub fn production(endpoint: T) -> Self {
        Self::new(ApiBase::production(), endpoint)
    }

    /// Creates a new client configured to use the staging API base.
    pub fn staging(endpoint: T) -> Self {
        Self::new(ApiBase::staging(), endpoint)
    }

    /// Converts the configured endpoint into a [`url::Url`].
    ///
    /// # Returns
    /// - `Ok(url::Url)` if the URL was successfully built.
    /// - `Err(url::ParseError)` if the URL construction failed.
    pub fn try_as_url(&self) -> Result<url::Url, Error> {
        self.endpoint.try_as_url(&self.base)
    }
}

impl<T: TryAsUrl + GetRequest> Adoptium<T> {
    /// Executes the GET request for the configured endpoint.
    ///
    /// This method is available only if the endpoint implements [`GetRequest`].
    ///
    /// # Returns
    /// - `Ok(T::Response)` if the request succeeded and the response was deserialized.
    /// - `Err(Error)` if the request or deserialization failed.
    pub async fn get(&self) -> Result<T::Response, Error> {
        self.endpoint.get(&self.base).await
    }
}

#[cfg(test)]
mod tests {
    use crate::v3::{adoptium::{ApiBase, Error, TryAsUrl}, prelude::Adoptium};

    struct MySuccessEndpoint {
        id: u8,
    }

    impl TryAsUrl for MySuccessEndpoint {
        fn try_as_url(&self, base: &ApiBase) -> Result<url::Url, Error> {
            let stringified_url = format!("{base}/{id}", id = self.id);

            url::Url::parse(&stringified_url)
                .map_err(Into::into)
        }
    }

    struct MyFailEndpoint {
        id: u8,
    }

    impl TryAsUrl for MyFailEndpoint {
        fn try_as_url(&self, _base: &ApiBase) -> Result<url::Url, Error> {
            let stringified_url = format!("/{id}", id = self.id);

            url::Url::parse(&stringified_url)
                .map_err(Into::into)
        }
    }

    #[test]
    fn try_into_url_success() {
        let endpoint = Adoptium::production(MySuccessEndpoint { id: 8 });

        let expected = url::Url::parse(&format!("{base}/8", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url()
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn try_into_url_fail() {
        let endpoint = Adoptium::production(MyFailEndpoint { id: 8 });

        let expected = url::ParseError::RelativeUrlWithoutBase;

        let provided = endpoint.try_as_url()
            .expect_err("Must fail to parse provided value");

        match provided {
            Error::Url(provided) => assert_eq!(expected, provided),
            other => panic!("Expected Url error, got {:?}", other),
        }
    }
}
