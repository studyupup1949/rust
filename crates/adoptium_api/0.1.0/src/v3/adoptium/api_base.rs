//! Provides the [`ApiBase`] type for selecting the Adoptium API environment.
//!
//! This module defines a small wrapper around API base URLs,
//! allowing code to easily switch between production and staging endpoints.

/// A wrapper around an API base URL for Adoptium.
///
/// By default, [`ApiBase`] uses the production endpoint.
#[derive(Debug)]
pub struct ApiBase(&'static str);

impl ApiBase {
    const BASE_URL: &str = "https://api.adoptium.net";
    const STAGING_BASE_URL: &str = "https://staging-api.adoptium.net";

    /// Returns an [`ApiBase`] set to the production endpoint.
    ///
    /// BASE: `https://api.adoptium.net`
    pub fn production() -> Self {
        Self(Self::BASE_URL)
    }

    /// Returns an [`ApiBase`] set to the staging endpoint.
    ///
    /// BASE: `https://staging-api.adoptium.net`
    pub fn staging() -> Self {
        Self(Self::STAGING_BASE_URL)
    }
}

impl Default for ApiBase {
    fn default() -> Self {
        Self::production()
    }
}

impl From<ApiBase> for &'static str {
    fn from(val: ApiBase) -> Self {
        val.0
    }
}

impl std::fmt::Display for ApiBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::v3::adoptium::ApiBase;

    #[test]
    fn production_base() {
        let expected = "https://api.adoptium.net";
        let provided = ApiBase::production().to_string();

        assert_eq!(expected, provided);
    }

    #[test]
    fn staging_base() {
        let expected = "https://staging-api.adoptium.net";
        let provided = ApiBase::staging().to_string();

        assert_eq!(expected, provided);
    }
}
