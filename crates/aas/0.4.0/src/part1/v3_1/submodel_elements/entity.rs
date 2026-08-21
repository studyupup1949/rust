use crate::part1::ToJsonMetamodel;
use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::core::SpecificAssetId;
use crate::part1::v3_1::primitives::Identifier;
use crate::part1::v3_1::submodel_elements::SubmodelElement;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// The entity submodel element is designed to be used in submodels defining the relationship between the parts of the composite asset
/// it is composed of (e.g. bill of material).
/// These parts are called entities. Not all entities have a global asset ID.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Display, EnumString)]
#[serde(tag = "entityType")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::EntityXMLProxy", into = "xml::EntityXMLProxy")
)]
pub enum Entity {
    /// There is no separate Asset Administration Shell for co-managed entities.
    /// Co-managed entities need to be part of a self-managed entity.
    CoManagedEntity(EntityInner),

    /// Self-managed entities have their own Asset Administration Shell but can be part of another composite self-managed entity.
    /// The asset represented by an Asset Administration Shell is a self-managed entity per definition
    SelfManagedEntity(EntityInner),
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct EntityInner {
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
    /// Statement applicable to the entity,
    /// each statement described by submodel element - typically with a qualified value
    pub statements: Option<Vec<SubmodelElement>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "globalAssetId")]
    pub global_asset_id: Option<Identifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "specificAssetId")]
    pub specific_asset_id: Option<Vec<SpecificAssetId>>,
}

impl ToJsonMetamodel for Entity {
    type Error = ();

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        // TODO: Add modelType
        match self {
            Entity::CoManagedEntity(_) => Ok(r#"{"entityType":"CoManagedEntity"}"#.into()),
            Entity::SelfManagedEntity(_) => Ok(r#"{"entityType":"SelfManagedEntity"}"#.into()),
        }
    }
}

pub(crate) mod xml {
    use crate::part1::v3_1::attributes::data_specification::{
        EmbeddedDataSpecification, HasDataSpecification,
    };
    use crate::part1::v3_1::attributes::extension::{Extension, HasExtensions};
    use crate::part1::v3_1::attributes::qualifiable::{Qualifiable, Qualifier};
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::core::SpecificAssetId;
    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::reference::Reference;
    use crate::part1::v3_1::submodel_elements::SubmodelElement;
    use crate::part1::v3_1::submodel_elements::entity::{Entity, EntityInner};
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub enum EntityType {
        CoManagedEntity,
        SelfManagedEntity,
    }

    #[derive(Serialize, Deserialize)]
    pub struct EntityXMLProxy {
        // Inherited from DataElement
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

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "semanticId")]
        pub semantic_id: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "supplementalSemanticIds")]
        pub supplemental_semantic_ids: Option<Vec<Reference>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub qualifiers: Option<Vec<Qualifier>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,

        /// Statement applicable to the entity,
        /// each statement described by submodel element - typically with a qualified value
        pub statements: Option<Vec<SubmodelElement>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "globalAssetId")]
        pub global_asset_id: Option<Identifier>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "specificAssetId")]
        pub specific_asset_id: Option<Vec<SpecificAssetId>>,

        #[serde(rename = "entityType")]
        pub ty: EntityType,
    }

    impl From<super::Entity> for EntityXMLProxy {
        fn from(value: super::Entity) -> Self {
            match value {
                Entity::CoManagedEntity(value) => Self {
                    id_short: value.referable.id_short,
                    display_name: value
                        .referable
                        .display_name
                        .map(|values| LangStringTextType { values }),
                    description: value
                        .referable
                        .description
                        .map(|values| LangStringTextType { values }),
                    #[allow(deprecated)]
                    category: value.referable.category,
                    extension: value.referable.extensions.extension,
                    semantic_id: value.semantics.semantic_id,
                    supplemental_semantic_ids: value.semantics.supplemental_semantic_ids,
                    qualifiers: value.qualifiable.qualifiers,
                    embedded_data_specifications: value
                        .embedded_data_specifications
                        .embedded_data_specifications,
                    statements: value.statements,
                    global_asset_id: value.global_asset_id,
                    specific_asset_id: value.specific_asset_id,
                    ty: EntityType::CoManagedEntity,
                },
                Entity::SelfManagedEntity(value) => Self {
                    id_short: value.referable.id_short,
                    display_name: value
                        .referable
                        .display_name
                        .map(|values| LangStringTextType { values }),
                    description: value
                        .referable
                        .description
                        .map(|values| LangStringTextType { values }),
                    #[allow(deprecated)]
                    category: value.referable.category,
                    extension: value.referable.extensions.extension,
                    semantic_id: value.semantics.semantic_id,
                    supplemental_semantic_ids: value.semantics.supplemental_semantic_ids,
                    qualifiers: value.qualifiable.qualifiers,
                    embedded_data_specifications: value
                        .embedded_data_specifications
                        .embedded_data_specifications,
                    statements: value.statements,
                    global_asset_id: value.global_asset_id,
                    specific_asset_id: value.specific_asset_id,
                    ty: EntityType::SelfManagedEntity,
                },
            }
        }
    }

    impl From<EntityXMLProxy> for super::Entity {
        fn from(value: EntityXMLProxy) -> Self {
            let inner = EntityInner {
                referable: Referable {
                    id_short: value.id_short,
                    display_name: value.display_name.map(LangStringTextType::into),
                    description: value.description.map(LangStringTextType::into),
                    #[allow(deprecated)]
                    category: value.category,
                    extensions: HasExtensions {
                        extension: value.extension,
                    },
                },
                semantics: HasSemantics {
                    semantic_id: value.semantic_id,
                    supplemental_semantic_ids: value.supplemental_semantic_ids,
                },
                qualifiable: Qualifiable {
                    qualifiers: value.qualifiers,
                },
                embedded_data_specifications: HasDataSpecification {
                    embedded_data_specifications: value.embedded_data_specifications,
                },

                statements: value.statements,
                global_asset_id: value.global_asset_id,
                specific_asset_id: value.specific_asset_id,
            };

            match value.ty {
                EntityType::CoManagedEntity => Self::CoManagedEntity(inner),
                EntityType::SelfManagedEntity => Self::SelfManagedEntity(inner),
            }
        }
    }
}

#[cfg(all(feature = "json", test))]
mod tests {
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::submodel_elements::entity::{Entity, EntityInner};

    #[test]
    fn deserialize_simple() {
        let json = r#"
        {
            "idShort": "Example1",
            "entityType": "SelfManagedEntity",
            "globalAssetId": "https://example.com"
        }"#;

        let expected = Entity::SelfManagedEntity(EntityInner {
            referable: Referable {
                id_short: Some(Identifier::try_from("Example1").unwrap()),
                ..Default::default()
            },
            global_asset_id: Some(Identifier::try_from("https://example.com").unwrap()),
            ..Default::default()
        });

        let actual = serde_json::from_str::<Entity>(json).unwrap();
        assert_eq!(actual, expected)
    }
}
