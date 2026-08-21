use crate::{api_request, v3::responses::types::{ArchitecturesResponse, OperatingSystemsResponse}};


api_request! {
    /// GET: `/v3/types/architectures`
    ///
    /// Returns names of architectures
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Types/getArchitectures>
    struct Architectures {
        endpoint: "/v3/types/architectures",
        request: Get => ArchitecturesResponse,
    }
}

api_request! {
    /// GET: `/v3/types/operating_systems`
    ///
    /// Returns names of operating systems
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Types/getOperatingSystems>
    struct OperationgSystems {
        endpoint: "/v3/types/operating_systems",
        request: Get => OperatingSystemsResponse,
    }
}

#[cfg(test)]
mod tests {
    use crate::v3::adoptium::{ApiBase, TryAsUrl};

    #[test]
    fn architectures() {
        let endpoint = super::Architectures::new();

        let expected = url::Url::parse(&format!("{base}/v3/types/architectures", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn operating_systems() {
        let endpoint = super::OperationgSystems::new();

        let expected = url::Url::parse(&format!("{base}/v3/types/operating_systems", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }
}
