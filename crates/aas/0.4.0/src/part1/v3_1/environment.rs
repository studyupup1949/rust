use crate::part1::v3_1::concept_description::ConceptDescription;
use crate::part1::v3_1::core::{AssetAdministrationShell, Submodel};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::EnvironmentXMLProxy", into = "xml::EnvironmentXMLProxy")
)]
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

#[cfg(feature = "xml")]
mod xml {
    use crate::part1::v3_1::concept_description::ConceptDescription;
    use crate::part1::v3_1::core::{AssetAdministrationShell, Submodel};
    use crate::part1::v3_1::environment::Environment;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AssetAdministrationShellWrapper {
        #[serde(rename = "assetAdministrationShell")]
        values: Option<Vec<AssetAdministrationShell>>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SubmodelWrapper {
        #[serde(rename = "submodel")]
        values: Option<Vec<Submodel>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ConceptDescriptionWrapper {
        #[serde(rename = "conceptDescription")]
        values: Option<Vec<ConceptDescription>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EnvironmentXMLProxy {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "assetAdministrationShells")]
        pub asset_administration_shells: Option<AssetAdministrationShellWrapper>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub submodels: Option<SubmodelWrapper>,

        #[serde(rename = "conceptDescriptions")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub concept_descriptions: Option<ConceptDescriptionWrapper>,
    }

    impl From<Environment> for EnvironmentXMLProxy {
        fn from(value: Environment) -> Self {
            Self {
                asset_administration_shells: Some(AssetAdministrationShellWrapper {
                    values: value.asset_administration_shells,
                }),
                submodels: Some(SubmodelWrapper {
                    values: value.submodels,
                }),
                concept_descriptions: Some(ConceptDescriptionWrapper {
                    values: value.concept_descriptions,
                }),
            }
        }
    }
    impl From<EnvironmentXMLProxy> for Environment {
        fn from(value: EnvironmentXMLProxy) -> Self {
            Self {
                asset_administration_shells: value
                    .asset_administration_shells
                    .and_then(|v| v.values),
                submodels: value.submodels.and_then(|v| v.values),
                concept_descriptions: value.concept_descriptions.and_then(|v| v.values),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::part1::v3_1::environment::Environment;

        #[test]
        fn deserialize_xml_mvp_dpp() {
            let xml = include_str!("../../../tests/mvp-dpp-1.0.0.xml");

            let env: Environment = quick_xml::de::from_str(xml).expect("Deserialize works");
            println!("{:#?}", env);
        }
    }
}

#[cfg(not(feature = "xml"))]
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
