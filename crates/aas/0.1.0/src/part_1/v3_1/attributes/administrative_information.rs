use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::reference::Reference;
use serde::de::{self};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use crate::part_1::v3_1::primitives::Identifier;

/// Administrative metainformation for an element like version information
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Debug, Serialize)]
struct Version {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

#[derive(Debug, Error)]
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
