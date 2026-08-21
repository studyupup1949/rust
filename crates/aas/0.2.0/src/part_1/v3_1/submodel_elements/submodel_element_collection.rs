use crate::part_1::ToJsonMetamodel;
use crate::part_1::v3_1::submodel_elements::SubmodelElement;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SubmodelElementCollection {
    value: Option<Vec<SubmodelElement>>,
}

impl ToJsonMetamodel for SubmodelElementCollection {
    type Error = ();

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        Ok(format!(r#"{{"modelType":"SubmodelElementCollection"}}"#))
    }
}
