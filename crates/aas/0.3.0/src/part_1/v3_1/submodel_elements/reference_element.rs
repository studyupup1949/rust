use crate::part_1::ToJsonMetamodel;
use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// ToJsonMetadata implemented from upper enum.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReferenceElement {
    /// External reference to an external object or entity or a logical reference
    /// to another element within the same or another Asset Administration Shell
    /// (i.e. a model reference to a Referable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Reference>,
}

impl ToJsonMetamodel for ReferenceElement {
    type Error = ();

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        Ok(format!(r#"{{"modelType":"ReferenceElement"}}"#))
    }
}
