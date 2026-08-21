use crate::part_1::v3_1::LangString;
use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part_1::v3_1::attributes::referable::Referable;
use crate::part_1::v3_1::attributes::semantics::HasSemantics;
use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct MultiLanguageProperty {
    // Inherited from DataElement
    #[serde(flatten)]
    pub referable: Referable,

    #[serde(flatten)]
    pub semantics: HasSemantics,

    #[serde(flatten)]
    pub qualifiable: Qualifiable,

    #[serde(flatten)]
    pub embedded_data_specifications: HasDataSpecification,
    // ----- end inheritance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<LangString>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "valueId")]
    pub value_id: Option<Reference>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn it_serializes() {
        let expected = r#"{"value":[{"language":"de","text":"Das ist ein deutscher Bezeichner"},{"language":"en","text":"That's an English label"}]}"#;

        let mut ml_property = MultiLanguageProperty::default();
        ml_property.value = Some(vec![
            LangString::from_str(r#""Das ist ein deutscher Bezeichner"@de"#).unwrap(),
            LangString::from_str(r#""That's an English label"@en"#).unwrap(),
        ]);

        let actual =
            serde_json::to_string(&ml_property).expect("Can't serialize MultiLanguageProperty.");

        assert_eq!(actual, expected);
    }
}
