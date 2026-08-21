use crate::{api_request, v3::responses::version::VersionResponse};


api_request! {
    /// GET: `/v3/version/{version}`
    ///
    /// Parses a java version string
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Version/parseVersion>
    struct Version {
        endpoint: "/v3/version/{version}",
        request: Get => VersionResponse,

        path: {
            version: String,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::v3::adoptium::{ApiBase, TryAsUrl};

    #[test]
    fn version_url() {
        let endpoint = super::Version::new("8".to_string());

        let expected = url::Url::parse(&format!("{base}/v3/version/8", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }
}
