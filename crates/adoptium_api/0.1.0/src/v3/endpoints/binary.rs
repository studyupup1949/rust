use crate::{api_request, v3::primitives::prelude::*};


api_request! {
    /// GET: `/v3/binary/latest/{feature_version}/{release_type}/{os}/{arch}/{image_type}/{jvm_impl}/{heap_size}/{vendor}`
    ///
    /// Redirects to the binary that matches your current query
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Binary/getBinary>
    struct Latest {
        endpoint: "/v3/binary/latest/{feature_version}/{release_type}/{os}/{arch}/{image_type}/{jvm_impl}/{heap_size}/{vendor}",

        path: {
            feature_version: u8,
            release_type: ReleaseType,
            os: OperatingSystem,
            arch: Architecture,
            image_type: ImageType,
            jvm_impl: JvmImpl,
            heap_size: HeapSize,
            vendor: Vendor,
        }

        query: {
            c_lib: CLib,
            project: Project,
        }
    }
}

api_request! {
    /// GET: `/v3/binary/version/{release_name}/{os}/{arch}/{image_type}/{jvm_impl}/{heap_size}/{vendor}`
    ///
    /// Redirects to the binary that matches your current query
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Binary/getBinaryByVersion>
    struct Version {
        endpoint: "/v3/binary/version/{release_name}/{os}/{arch}/{image_type}/{jvm_impl}/{heap_size}/{vendor}",

        path: {
            release_name: String,
            os: OperatingSystem,
            arch: Architecture,
            image_type: ImageType,
            jvm_impl: JvmImpl,
            heap_size: HeapSize,
            vendor: Vendor,
        }

        query: {
            c_lib: CLib,
            project: Project,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::v3::adoptium::{ApiBase, TryAsUrl};

    #[test]
    fn latest_url_without_query() {
        let endpoint = super::Latest::new(
            8,
            crate::v3::prelude::ReleaseType::Ga,
            crate::v3::prelude::OperatingSystem::Linux,
            crate::v3::prelude::Architecture::X64,
            crate::v3::prelude::ImageType::Jdk,
            crate::v3::prelude::JvmImpl::Hotspot,
            crate::v3::prelude::HeapSize::Normal,
            crate::v3::prelude::Vendor::Eclipse,
        );

        let expected = url::Url::parse(&format!("{base}/v3/binary/latest/8/ga/linux/x64/jdk/hotspot/normal/eclipse", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn latest_url_with_query() {
        let endpoint = super::Latest::new(
            8,
            crate::v3::prelude::ReleaseType::Ea,
            crate::v3::prelude::OperatingSystem::Linux,
            crate::v3::prelude::Architecture::X64,
            crate::v3::prelude::ImageType::Jdk,
            crate::v3::prelude::JvmImpl::Hotspot,
            crate::v3::prelude::HeapSize::Normal,
            crate::v3::prelude::Vendor::Eclipse,
        ).c_lib(crate::v3::prelude::CLib::Musl).project(crate::v3::prelude::Project::Jdk);

        let mut expected = url::Url::parse(&format!("{base}/v3/binary/latest/8/ea/linux/x64/jdk/hotspot/normal/eclipse", base = ApiBase::production()))
            .expect("Failed to create expected value");

        {
            let mut query_pairs = expected.query_pairs_mut();
            query_pairs.append_pair("c_lib", "musl");
            query_pairs.append_pair("project", "jdk");
        }

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn version_url_without_query() {
        let endpoint = super::Version::new(
            "8".to_string(),
            crate::v3::prelude::OperatingSystem::Linux,
            crate::v3::prelude::Architecture::X64,
            crate::v3::prelude::ImageType::Jdk,
            crate::v3::prelude::JvmImpl::Hotspot,
            crate::v3::prelude::HeapSize::Normal,
            crate::v3::prelude::Vendor::Eclipse,
        );

        let expected = url::Url::parse(&format!("{base}/v3/binary/version/8/linux/x64/jdk/hotspot/normal/eclipse", base = ApiBase::production()))
            .expect("Failed to create expected value");

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }

    #[test]
    fn version_url_with_query() {
        let endpoint = super::Version::new(
            "8".to_string(),
            crate::v3::prelude::OperatingSystem::Linux,
            crate::v3::prelude::Architecture::X64,
            crate::v3::prelude::ImageType::Jdk,
            crate::v3::prelude::JvmImpl::Hotspot,
            crate::v3::prelude::HeapSize::Normal,
            crate::v3::prelude::Vendor::Eclipse,
        ).c_lib(crate::v3::prelude::CLib::Musl).project(crate::v3::prelude::Project::Jdk);

        let mut expected = url::Url::parse(&format!("{base}/v3/binary/version/8/linux/x64/jdk/hotspot/normal/eclipse", base = ApiBase::production()))
            .expect("Failed to create expected value");

        {
            let mut query_pairs = expected.query_pairs_mut();
            query_pairs.append_pair("c_lib", "musl");
            query_pairs.append_pair("project", "jdk");
        }

        let provided = endpoint.try_as_url(&ApiBase::production())
            .expect("Failed to create provided value");

        assert_eq!(expected, provided);
    }
}
