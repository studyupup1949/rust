use crate::{api_request, v3::{responses::assets::{FeatureReleasesResponse, LatestResponse, ReleaseNameResponse, VersionResponse}, primitives::prelude::*}};


api_request! {
    /// GET: `/v3/assets/feature_releases/{feature_version}/{release_type}`
    ///
    /// Returns release information
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Assets/searchReleases>
    struct FeatureReleases {
        endpoint: "/v3/assets/feature_releases/{feature_version}/{release_type}",
        request: Get => FeatureReleasesResponse,

        path: {
            feature_version: u8,
            release_type: ReleaseType,
        }

        query: {
            architecture: Architecture,
            before: String,
            c_lib: CLib,
            heap_size: HeapSize,
            image_type: ImageType,
            jvm_impl: JvmImpl,
            os: OperatingSystem,
            page: u8,
            page_size: u8,
            project: Project,
            sort_method: SortMethod,
            sort_order: SortOrder,
            vendor: Vendor,
        }
    }
}

api_request! {
    /// GET: `/v3/assets/latest/{feature_version}/{jvm_impl}`
    ///
    /// Returns list of latest assets for the given feature version and jvm impl
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Assets/getLatestAssets>
    struct Latest {
        endpoint: "/v3/assets/latest/{feature_version}/{jvm_impl}",
        request: Get => LatestResponse,

        path: {
            feature_version: u8,
            jvm_impl: JvmImpl,
        }

        query: {
            architecture: Architecture,
            image_type: ImageType,
            os: OperatingSystem,
            vendor: Vendor,
        }
    }
}

api_request! {
    /// GET: `/v3/assets/release_name/{vendor}/{release_name}`
    ///
    /// Returns release information
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Assets/getReleaseInfo>
    struct ReleaseName {
        endpoint: "/v3/assets/release_name/{vendor}/{release_name}",
        request: Get => ReleaseNameResponse,

        path: {
            vendor: Vendor,
            release_name: String,
        }

        query: {
            architecture: Architecture,
            c_lib: CLib,
            heap_size: HeapSize,
            image_type: ImageType,
            jvm_impl: JvmImpl,
            os: OperatingSystem,
            project: Project,
        }
    }
}

api_request! {
    /// GET: `/v3/assets/version/{version}`
    ///
    /// Returns release information about the specified version.
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Assets/searchReleasesByVersion>
    struct Version {
        endpoint: "/v3/assets/version/{version}",
        request: Get => VersionResponse,

        path: {
            version: String,
        }

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
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::v3::adoptium::{ApiBase, TryAsUrl};

    #[test]
    fn feature_releases_url_without_query() {
        let endpoint = super::FeatureReleases::new(8, crate::v3::prelude::ReleaseType::Ga);

        let expected = url::Url::parse(&format!("{base}/v3/assets/feature_releases/8/ga", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn feature_releases_url_with_query() {
        let endpoint = super::FeatureReleases::new(8, crate::v3::prelude::ReleaseType::Ga)
            .before("8".to_string())
            .c_lib(crate::v3::prelude::CLib::Glibc);

        let mut expected = url::Url::parse(&format!("{base}/v3/assets/feature_releases/8/ga", base = ApiBase::production()))
            .expect("Failed to create expected value");

        {
            let mut query_pairs = expected.query_pairs_mut();
            query_pairs.append_pair("before", "8");
            query_pairs.append_pair("c_lib", "glibc");
        }

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn latest_url_without_query() {
        let endpoint = super::Latest::new(8, crate::v3::prelude::JvmImpl::Hotspot);

        let expected = url::Url::parse(&format!("{base}/v3/assets/latest/8/hotspot", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn latest_url_with_query() {
        let endpoint = super::Latest::new(8, crate::v3::prelude::JvmImpl::Hotspot)
            .os(crate::v3::prelude::OperatingSystem::Windows);

        let mut expected = url::Url::parse(&format!("{base}/v3/assets/latest/8/hotspot", base = ApiBase::production()))
            .expect("Failed to create expected value");

        {
            let mut query_pairs = expected.query_pairs_mut();
            query_pairs.append_pair("os", "windows");
        }

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn release_name_url_without_query() {
        let endpoint = super::ReleaseName::new(crate::v3::prelude::Vendor::Eclipse, "8".to_string());

        let expected = url::Url::parse(&format!("{base}/v3/assets/release_name/eclipse/8", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn reelase_name_url_with_query() {
        let endpoint = super::ReleaseName::new(crate::v3::prelude::Vendor::Eclipse, "8".to_string())
            .project(crate::v3::prelude::Project::Valhalla);

        let mut expected = url::Url::parse(&format!("{base}/v3/assets/release_name/eclipse/8", base = ApiBase::production()))
            .expect("Failed to create expected value");

        {
            let mut query_pairs = expected.query_pairs_mut();
            query_pairs.append_pair("project", "valhalla");
        }

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn version_url_without_query() {
        let endpoint = super::Version::new("8".to_string());

        let expected = url::Url::parse(&format!("{base}/v3/assets/version/8", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn version_url_with_query() {
        let endpoint = super::Version::new("8".to_string())
            .lts(true);

        let mut expected = url::Url::parse(&format!("{base}/v3/assets/version/8", base = ApiBase::production()))
            .expect("Failed to create expected value");

        {
            let mut query_pairs = expected.query_pairs_mut();
            query_pairs.append_pair("lts", "true");
        }

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }
}
