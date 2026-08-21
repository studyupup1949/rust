use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::primitives::Identifier;
use crate::part1::v3_1::reference::Reference;
use serde::de::{self};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Administrative metainformation for an element like version information
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(
        from = "xml::AdministrativeInformationXML",
        into = "xml::AdministrativeInformationXML"
    )
)]
pub struct AdministrativeInformation {
    #[serde(flatten)]
    pub version: Version,

    /// The subject ID of the subject responsible for making the element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<Reference>,

    #[serde(rename = "templateId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<Identifier>,

    #[serde(flatten)]
    pub data_specification: HasDataSpecification,
}

/// Versioning
/// Constraint AASd-005:
/// If AdministrativeInformation/version is not specified,
/// AdministrativeInformation/revision shall also be unspecified.
/// This means that a revision requires a version.
/// If there is no version, there is no revision.
/// Revision is optional.
#[derive(Clone, PartialEq, Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Version {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum VersionError {
    #[error("Revision can not exist without version")]
    RevisionNotApplicable,
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawVersion {
            version: Option<String>,
            // TODO MAX length of 3
            revision: Option<String>,
        }

        let raw = RawVersion::deserialize(deserializer)?;

        if raw.revision.is_some() && raw.version.is_none() {
            return Err(de::Error::custom(
                VersionError::RevisionNotApplicable.to_string(),
            ));
        }

        Ok(Version {
            version: raw.version,
            revision: raw.revision,
        })
    }
}

mod xml {
    use crate::part1::v3_1::attributes::administrative_information::{
        AdministrativeInformation, Version,
    };
    use crate::part1::v3_1::attributes::data_specification::{
        EmbeddedDataSpecification, HasDataSpecification,
    };
    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::reference::Reference;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub(super) struct AdministrativeInformationXML {
        version: Option<String>,
        revision: Option<String>,

        /// The subject ID of the subject responsible for making the element
        #[serde(skip_serializing_if = "Option::is_none")]
        pub creator: Option<Reference>,

        #[serde(rename = "templateId")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub template_id: Option<Identifier>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        pub embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,
    }

    /*impl<'de> Deserialize<'de> for AdministrativeInformationXML {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let ai = AdministrativeInformationXML::deserialize(deserializer)?;

            // TODO: Max Length of 3
            if ai.revision.is_some() && ai.version.is_none() {
                return Err(de::Error::custom(
                    VersionError::RevisionNotApplicable.to_string(),
                ));
            }

            Ok(ai)
        }
    }*/

    impl From<AdministrativeInformationXML> for AdministrativeInformation {
        fn from(value: AdministrativeInformationXML) -> Self {
            Self {
                version: Version {
                    version: value.version,
                    revision: value.revision,
                },
                creator: value.creator,
                template_id: value.template_id,
                data_specification: HasDataSpecification {
                    embedded_data_specifications: value.embedded_data_specifications,
                },
            }
        }
    }
    impl From<AdministrativeInformation> for AdministrativeInformationXML {
        fn from(value: AdministrativeInformation) -> Self {
            Self {
                version: value.version.version,
                revision: value.version.revision,
                creator: value.creator,
                template_id: value.template_id,
                embedded_data_specifications: value.data_specification.embedded_data_specifications,
            }
        }
    }
}
