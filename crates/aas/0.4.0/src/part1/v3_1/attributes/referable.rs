use crate::part1::v3_1::attributes::extension::HasExtensions;
use crate::part1::v3_1::primitives::{Identifier, MultiLanguageNameType};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use crate::utilities::deserialize_empty_identifier_as_none;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::ReferableXMLProxy", into = "xml::ReferableXMLProxy")
)]
pub struct Referable {
    #[serde(skip_serializing_if = "Option::is_none")]
    // use case where "" is needed or can this be ignored?
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_empty_identifier_as_none")]
    #[serde(rename = "idShort")]
    pub id_short: Option<Identifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "displayName")]
    pub display_name: Option<MultiLanguageNameType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<MultiLanguageNameType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated]
    pub category: Option<String>,

    /// HasExtensions
    #[serde(flatten)]
    pub extensions: HasExtensions,
}

pub(crate) mod xml {
    use crate::part1::v3_1::attributes::extension::{Extension, HasExtensions};
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Default)]
    pub struct ReferableXMLProxy {
        #[serde(skip_serializing_if = "Option::is_none")]
        // use case where "" is needed or can this be ignored?
        #[serde(default)]
        #[serde(deserialize_with = "deserialize_empty_identifier_as_none")]
        #[serde(rename = "idShort")]
        pub id_short: Option<Identifier>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "displayName")]
        pub display_name: Option<LangStringTextType>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<LangStringTextType>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[deprecated]
        pub category: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "extensions")]
        pub extension: Option<Vec<Extension>>,
    }

    impl From<Referable> for ReferableXMLProxy {
        fn from(value: Referable) -> Self {
            Self {
                id_short: value.id_short,
                display_name: value
                    .display_name
                    .map(|values| LangStringTextType { values }),
                description: value
                    .description
                    .map(|values| LangStringTextType { values }),
                #[allow(deprecated)]
                category: value.category,
                extension: value.extensions.extension,
            }
        }
    }
    impl From<ReferableXMLProxy> for Referable {
        fn from(value: ReferableXMLProxy) -> Self {
            Self {
                id_short: value.id_short,
                display_name: value.display_name.map(LangStringTextType::into),
                description: value.description.map(LangStringTextType::into),
                #[allow(deprecated)]
                category: value.category,
                extensions: HasExtensions {
                    extension: value.extension,
                },
            }
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn deserialize_referable_from_xml() {}
    }
}
