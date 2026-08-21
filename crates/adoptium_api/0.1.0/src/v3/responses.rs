pub mod assets {
    use crate::v3::primitives::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct FeatureReleasesResponse(pub Vec<Release>);

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct LatestResponse(pub Vec<BinaryAssetView>);

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct ReleaseNameResponse(pub Vec<Release>);

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct VersionResponse(pub Vec<Release>);
}

pub mod release_info {
    use crate::v3::primitives::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct AvailableReleasesResponse {
        pub available_releases: Vec<u8>,
        pub available_lts_releases: Vec<u8>,
        pub most_recent_lts: u8,
        pub most_recent_feature_release: u8,
        pub most_recent_feature_version: u8,
        pub tip_version:u8,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct ReleaseNamesResponse {
        pub releases: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct ReleaseNotesResponse {
        pub version_data: VersionData,
        pub vendor: Option<Vendor>,
        pub id: String,
        pub release_name: String,
        pub release_notes: Vec<ReleaseNote>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct ReleaseVersionsResponse {
        pub versions: Vec<VersionData>,
    }
}

pub mod types {
    use crate::v3::primitives::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct ArchitecturesResponse(pub Vec<Architecture>);

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct OperatingSystemsResponse(pub Vec<OperatingSystem>);
}

pub mod version {
    use crate::v3::primitives::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct VersionResponse(pub VersionData);
}
