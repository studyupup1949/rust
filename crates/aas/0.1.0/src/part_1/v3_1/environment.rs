use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::identifiable::Identifiable;
use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
use crate::part_1::v3_1::core::{AssetAdministrationShell, Submodel};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Environment {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "assetAdministrationShells")]
    pub asset_administration_shells: Option<Vec<AssetAdministrationShell>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub submodels: Option<Vec<Submodel>>,

    #[serde(rename = "conceptDescriptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concept_descriptions: Option<Vec<ConceptDescription>>,
}

/// The semantics of a property or other elements that may have a semantic description is defined
/// by a concept description.
/// The description of the concept should follow a standardized schema
/// (realized as data specification template).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ConceptDescription {
    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_specification: Option<HasDataSpecification>,

    #[serde(rename = "isCaseOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_case_of: Option<Reference>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize() {
        let json = include_str!("../../../tests/env.json");

        let env: Environment = serde_json::from_str(json).expect("Deserialize works");

        println!("{:#?}", env);
    }
}
