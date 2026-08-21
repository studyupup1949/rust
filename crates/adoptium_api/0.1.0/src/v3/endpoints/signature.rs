use crate::{api_request, v3::primitives::prelude::*};


api_request! {
    /// GET: `/v3/signature/version/{release_name}/{os}/{arch}/{image_type}/{jvm_impl}/{heap_size}/{vendor}`
    ///
    /// Redirects to the signature of the release that matches your current query
    ///
    /// <https://api.adoptium.net/q/swagger-ui/#/Signature/getSignatureByVersion>
    struct Version {
        endpoint: "/v3/signature/version/{release_name}/{os}/{arch}/{image_type}/{jvm_impl}/{heap_size}/{vendor}",

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

        let expected = url::Url::parse(&format!("{base}/v3/signature/version/8/linux/x64/jdk/hotspot/normal/eclipse", base = ApiBase::production()))
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

        let mut expected = url::Url::parse(&format!("{base}/v3/signature/version/8/linux/x64/jdk/hotspot/normal/eclipse", base = ApiBase::production()))
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
