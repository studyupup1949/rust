pub mod prelude {
    pub use super::adoptium_jvm_impl::AdoptiumJvmImpl;
    pub use super::adoptium_vendor::AdoptiumVendor;
    pub use super::architecture::Architecture;
    pub use super::binary::Binary;
    pub use super::binary_asset_view::BinaryAssetView;
    pub use super::c_lib::CLib;
    pub use super::heap_size::HeapSize;
    pub use super::image_type::ImageType;
    pub use super::installer::Installer;
    pub use super::jvm_impl::JvmImpl;
    pub use super::operating_system::OperatingSystem;
    pub use super::package::Package;
    pub use super::release::Release;
    pub use super::release_note::ReleaseNote;
    pub use super::release_notes_package::ReleaseNotesPackage;
    pub use super::release_type::ReleaseType;
    pub use super::sort_method::SortMethod;
    pub use super::sort_order::SortOrder;
    pub use super::source_package::SourcePackage;
    pub use super::stats_source::StatsSource;
    pub use super::vendor::Vendor;
    pub use super::version_data::VersionData;
    pub use super::project::Project;
}

pub mod adoptium_jvm_impl {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum AdoptiumJvmImpl {
        #[default] Hotspot,
    }
}

pub mod adoptium_vendor {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum AdoptiumVendor {
        #[default] Eclipse,
    }
}

pub mod architecture {
    #[derive(Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum Architecture {
        X64,
        X86,
        X32,
        Ppc64,
        Ppc64le,
        S390x,
        Aarch64,
        Arm,
        Sparcv9,
        Riscv64,
    }
}

pub mod binary {
    use crate::v3::primitives::{architecture::Architecture, c_lib::CLib, heap_size::HeapSize, image_type::ImageType, installer::Installer, jvm_impl::JvmImpl, operating_system::OperatingSystem, package::Package, project::Project};

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct Binary {
        pub os: OperatingSystem,
        pub architecture: Architecture,
        pub image_type: ImageType,
        pub c_lib: Option<CLib>,
        pub jvm_impl: Option<JvmImpl>,
        pub package: Option<Package>,
        pub installer: Option<Installer>,
        pub heap_size: HeapSize,
        pub download_count: Option<i64>,
        pub updated_at: chrono::DateTime<chrono::Utc>, // FIXME: make sure about UTC
        pub scm_ref: Option<String>,
        pub project: Project,
    }
}

pub mod binary_asset_view {
    use crate::v3::primitives::{binary::Binary, vendor::Vendor, version_data::VersionData};

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct BinaryAssetView {
        pub binary: Option<Binary>,
        pub release_name: String,
        pub release_link: url::Url,
        pub vendor: Option<Vendor>,
        pub version: Option<VersionData>,
    }
}

pub mod c_lib {
    #[derive(Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum CLib {
        Musl,
        Glibc,
    }
}

pub mod heap_size {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum HeapSize {
        #[default] Normal,
        Large,
    }
}

pub mod image_type {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum ImageType {
        #[default] Jdk,
        Jre,
        Testimage,
        Debugimage,
        Staticlibs,
        Sources,
        Sbom,
        Jmods,
    }
}

pub mod installer {
    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct Installer {
        pub name: String,
        pub link: url::Url,
        pub size: Option<i64>,
        pub checksum: Option<String>,
        pub checksum_link: Option<url::Url>,
        pub signature_link: Option<url::Url>,
        pub download_count: Option<i64>,
        pub metadata_link: Option<url::Url>,
    }
}

pub mod jvm_impl {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum JvmImpl {
        #[default] Hotspot,
    }
}

pub mod operating_system {
    #[derive(Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum OperatingSystem {
        Linux,
        Windows,
        Mac,
        Solaris,
        Aix,
        #[strum(serialize = "alpine-linux")]
        #[serde(rename = "alpine-linux")]
        AlpineLinux,
    }
}

pub mod package {
    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct Package {
        pub name: String,
        pub link: url::Url,
        pub size: Option<i64>,
        pub checksum: Option<String>,
        pub checksum_link: Option<url::Url>,
        pub signature_link: Option<url::Url>,
        pub download_count: Option<i64>,
        pub metadata_link: Option<url::Url>,
    }
}

pub mod release {
    use crate::v3::primitives::{binary::Binary, release_notes_package::ReleaseNotesPackage, release_type::ReleaseType, source_package::SourcePackage, vendor::Vendor, version_data::VersionData};

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct Release {
        pub id: String,
        pub release_link: url::Url,
        pub release_name: String,
        pub timestamp: chrono::DateTime<chrono::Utc>, // FIXME: make sure about UTC
        pub updated_at: chrono::DateTime<chrono::Utc>, // FIXME: make sure about UTC
        pub binaries: Vec<Binary>,
        pub download_count: Option<i64>,
        pub release_type: ReleaseType,
        pub vendor: Option<Vendor>,
        pub version_data: VersionData,
        pub source: Option<SourcePackage>,
        pub release_notes: Option<ReleaseNotesPackage>,
        pub aqavit_results_link: Option<url::Url>,
    }
}

pub mod release_note {
    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct ReleaseNote {
        pub id: String,
        pub title: Option<String>,
        pub priority: Option<String>,
        pub component: Option<String>,
        pub subcomponent: Option<String>,
        pub link: Option<url::Url>,
        pub r#type: Option<String>,
        pub backport_of: Option<String>,
    }
}

pub mod release_notes_package {
    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct ReleaseNotesPackage {
        pub name: String,
        pub link: url::Url,
        pub size: Option<i64>,
    }
}

pub mod release_type {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum ReleaseType {
        #[default] Ga,
        Ea,
    }
}

pub mod sort_method {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum SortMethod {
        #[default] Default,
        Date,
    }
}

pub mod sort_order {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum SortOrder {
        Asc,
        #[default] Desc,
    }
}

pub mod source_package {
    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct SourcePackage {
        pub name: String,
        pub link: url::Url,
        pub size: Option<i64>,
    }
}

pub mod stats_source {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum StatsSource {
        Github,
        Dockerhub,
        #[default] All,
    }
}

pub mod vendor {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum Vendor {
        #[default] Eclipse,
    }
}

pub mod version_data {
    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct VersionData {
        pub major: Option<i32>,
        pub minor: Option<i32>,
        pub security: Option<i32>,
        pub patch: Option<i32>,
        pub pre: Option<String>,
        pub adopt_build_number: Option<i32>,
        pub semver: String,
        pub openjdk_version: String,
        pub build: Option<i32>,
        pub optional: Option<String>,
    }
}

pub mod project {
    #[derive(Default, Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display, serde::Deserialize, serde::Serialize)]
    #[strum(serialize_all = "lowercase")]
    #[serde(rename_all = "lowercase")]
    pub enum Project {
        #[default] Jdk,
        Valhalla,
        Metropolis,
        Jfr,
        Shenandoah,
    }
}
