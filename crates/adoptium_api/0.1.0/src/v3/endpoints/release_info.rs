use crate::{api_request, v3::{responses::release_info::{AvailableReleasesResponse, ReleaseNamesResponse, ReleaseNotesResponse, ReleaseVersionsResponse}, primitives::prelude::*}};


api_request! {
    /// GET: `/v3/info/available_releases`
    ///
    /// Returns information about available releases
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Release%20Info/getAvailableReleases>
    struct AvaiableReleases {
        endpoint: "/v3/info/available_releases",
        request: Get => AvailableReleasesResponse,
    }
}

api_request! {
    /// GET: `/v3/info/release_names`
    ///
    /// Returns a list of all release names
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Release%20Info/getReleaseNames>
    struct ReleaseNames {
        endpoint: "/v3/info/release_names",
        request: Get => ReleaseNamesResponse,

        query: {
            architecture: Architecture,
            c_lib: CLib,
            heap_size: HeapSize,
            image_type: ImageType,
            jvm_impl: JvmImpl,
            lts: bool,
            os: OperatingSystem,
            page: u8,
            page_size: u8,
            project: Project,
            release_type: ReleaseType,
            semver: bool,
            sort_method: SortMethod,
            sort_order: SortOrder,
            vendor: Vendor,
            version: String,
        }
    }
}

api_request! {
    /// GET: `/v3/info/release_notes/{release_name}`
    ///
    /// Returns release notes for a release version
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Release%20Info/getReleaseNotes>
    struct ReleaseNotes {
        endpoint: "/v3/info/release_notes/{release_name}",
        request: Get => ReleaseNotesResponse,

        path: {
            release_name: String,
        }

        query: {
            vendor: Vendor,
        }
    }
}

api_request! {
    /// GET: `/v3/info/release_versions`
    ///
    /// Returns a list of all release versions
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Release%20Info/getReleaseVersions>
    struct ReleaseVersions {
        endpoint: "/v3/info/release_versions",
        request: Get => ReleaseVersionsResponse,

        query: {
            architecture: Architecture,
            c_lib: CLib,
            heap_size: HeapSize,
            image_type: ImageType,
            jvm_impl: JvmImpl,
            lts: bool,
            os: OperatingSystem,
            page: u8,
            page_size: u8,
            project: Project,
            release_type: ReleaseType,
            semver: bool,
            sort_method: SortMethod,
            sort_order: SortOrder,
            vendor: Vendor,
            version: String,
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::v3::adoptium::{ApiBase, TryAsUrl};

    #[test]
    fn available_releases_url() {
        let endpoint = super::AvaiableReleases::new();

        let expected = url::Url::parse(&format!("{base}/v3/info/available_releases", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn release_names_url_without_query() {
        let endpoint = super::ReleaseNames::new();

        let expected = url::Url::parse(&format!("{base}/v3/info/release_names", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn release_names_url_with_query() {
        let endpoint = super::ReleaseNames::new()
            .architecture(crate::v3::prelude::Architecture::X64)
            .page(8);

        let mut expected = url::Url::parse(&format!("{base}/v3/info/release_names", base = ApiBase::production()))
            .expect("Failed to create expected value");

        {
            let mut query_pairs = expected.query_pairs_mut();
            query_pairs.append_pair("architecture", "x64");
            query_pairs.append_pair("page", "8");
        }

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn release_notes_url_without_query() {
        let endpoint = super::ReleaseNotes::new("8".to_string());

        let expected = url::Url::parse(&format!("{base}/v3/info/release_notes/8", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn release_notes_url_with_query() {
        let endpoint = super::ReleaseNotes::new("8".to_string())
            .vendor(crate::v3::prelude::Vendor::Eclipse);

        let mut expected = url::Url::parse(&format!("{base}/v3/info/release_notes/8", base = ApiBase::production()))
            .expect("Failed to create expected value");

        {
            let mut query_pairs = expected.query_pairs_mut();
            query_pairs.append_pair("vendor", "eclipse");
        }

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn release_versions_url_without_query() {
        let endpoint = super::ReleaseVersions::new();

        let expected = url::Url::parse(&format!("{base}/v3/info/release_versions", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn release_versions_url_with_query() {
        let endpoint = super::ReleaseVersions::new()
            .vendor(crate::v3::prelude::Vendor::Eclipse)
            .page_size(10)
            .semver(true)
            .version("8.0.0".to_string());

        let mut expected = url::Url::parse(&format!("{base}/v3/info/release_versions", base = ApiBase::production()))
            .expect("Failed to create expected value");

        {
            let mut query_pairs = expected.query_pairs_mut();
            query_pairs.append_pair("page_size", "10");
            query_pairs.append_pair("semver", "true");
            query_pairs.append_pair("vendor", "eclipse");
            query_pairs.append_pair("version", "8.0.0");
        }

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }
}
