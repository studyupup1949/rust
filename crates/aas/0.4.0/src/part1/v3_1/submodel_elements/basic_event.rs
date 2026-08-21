use crate::part1::v3_1::primitives::{DateTimeUTC, MessageTopic};
use crate::part1::v3_1::reference::Reference;
use crate::part1::v3_1::submodel_elements::SubmodelElementFields;
use crate::part1::{MetamodelError, ToJsonMetamodel};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::BasicEventElementXML", into = "xml::BasicEventElementXML")
)]
pub struct BasicEventElement {
    #[serde(flatten)]
    submodel_element_fields: SubmodelElementFields,

    pub observed: Reference,

    pub direction: Direction,

    pub state: StateOfEvent,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "messageTopic")]
    pub message_topic: Option<MessageTopic>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "messageBroker")]
    pub message_broker: Option<Reference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastUpdate")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    pub last_update: Option<DateTimeUTC>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "minInterval")]
    // TODO: duration type
    pub min_interval: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxInterval")]
    // TODO: duration type
    pub max_interval: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(
        from = "xml::BasicEventElementMetaXML",
        into = "xml::BasicEventElementMetaXML"
    )
)]
pub struct BasicEventElementMeta {
    #[serde(flatten)]
    submodel_element_fields: SubmodelElementFields,

    pub direction: Direction,

    pub state: StateOfEvent,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "messageTopic")]
    pub message_topic: Option<MessageTopic>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "messageBroker")]
    pub message_broker: Option<Reference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastUpdate")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    pub last_update: Option<DateTimeUTC>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "minInterval")]
    // TODO: duration type
    pub min_interval: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxInterval")]
    // TODO: duration type
    pub max_interval: Option<String>,
}

impl From<BasicEventElement> for BasicEventElementMeta {
    fn from(element: BasicEventElement) -> Self {
        Self {
            submodel_element_fields: element.submodel_element_fields,
            direction: element.direction,
            state: element.state,
            message_topic: element.message_topic,
            message_broker: element.message_broker,
            last_update: element.last_update,
            min_interval: element.min_interval,
            max_interval: element.max_interval,
        }
    }
}

impl From<&BasicEventElement> for BasicEventElementMeta {
    fn from(element: &BasicEventElement) -> Self {
        element.clone().into()
    }
}

impl ToJsonMetamodel for BasicEventElement {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        serde_json::to_string::<BasicEventElementMeta>(&self.into())
            .map_err(MetamodelError::FailedSerialisation)
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Display, EnumString)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum StateOfEvent {
    #[serde(rename = "on")]
    On,
    #[serde(rename = "off")]
    Off,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Display, EnumString)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Direction {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "output")]
    Output,
}

mod xml {
    use super::*;
    use crate::part1::v3_1::attributes::data_specification::{
        EmbeddedDataSpecification, HasDataSpecification,
    };
    use crate::part1::v3_1::attributes::extension::HasExtensions;
    use crate::part1::v3_1::attributes::qualifiable::{Qualifiable, Qualifier};
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::{
        part1::v3_1::{attributes::extension::Extension, primitives::Identifier},
        utilities::deserialize_empty_identifier_as_none,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
    pub struct BasicEventElementXML {
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

        pub observed: Reference,

        pub direction: Direction,

        pub state: StateOfEvent,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "messageTopic")]
        pub message_topic: Option<MessageTopic>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "messageBroker")]
        pub message_broker: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "lastUpdate")]
        pub last_update: Option<DateTimeUTC>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "minInterval")]
        // TODO: duration type
        pub min_interval: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "maxInterval")]
        // TODO: duration type
        pub max_interval: Option<String>,
    }

    impl From<BasicEventElementXML> for BasicEventElement {
        fn from(value: BasicEventElementXML) -> Self {
            Self {
                submodel_element_fields: SubmodelElementFields {
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
                },
                observed: value.observed,
                direction: value.direction,
                state: value.state,
                message_topic: value.message_topic,
                message_broker: value.message_broker,
                last_update: value.last_update,
                min_interval: value.min_interval,
                max_interval: value.max_interval,
            }
        }
    }
    impl From<BasicEventElement> for BasicEventElementXML {
        fn from(value: BasicEventElement) -> Self {
            Self {
                id_short: value.submodel_element_fields.referable.id_short,
                display_name: value
                    .submodel_element_fields
                    .referable
                    .display_name
                    .map(|values| LangStringTextType { values }),
                description: value
                    .submodel_element_fields
                    .referable
                    .description
                    .map(|values| LangStringTextType { values }),
                #[allow(deprecated)]
                category: value.submodel_element_fields.referable.category,
                extension: value.submodel_element_fields.referable.extensions.extension,
                semantic_id: value.submodel_element_fields.semantics.semantic_id,
                supplemental_semantic_ids: value
                    .submodel_element_fields
                    .semantics
                    .supplemental_semantic_ids,
                qualifiers: value.submodel_element_fields.qualifiable.qualifiers,
                embedded_data_specifications: value
                    .submodel_element_fields
                    .embedded_data_specifications
                    .embedded_data_specifications,
                direction: value.direction,
                observed: value.observed,
                state: value.state,
                message_topic: value.message_topic,
                message_broker: value.message_broker,
                last_update: value.last_update,
                min_interval: value.min_interval,
                max_interval: value.max_interval,
            }
        }
    }

    #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
    pub struct BasicEventElementMetaXML {
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

        pub direction: Direction,

        pub state: StateOfEvent,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "messageTopic")]
        pub message_topic: Option<MessageTopic>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "messageBroker")]
        pub message_broker: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "lastUpdate")]
        pub last_update: Option<DateTimeUTC>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "minInterval")]
        // TODO: duration type
        pub min_interval: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "maxInterval")]
        // TODO: duration type
        pub max_interval: Option<String>,
    }

    impl From<BasicEventElementMetaXML> for BasicEventElementMeta {
        fn from(value: BasicEventElementMetaXML) -> Self {
            Self {
                submodel_element_fields: SubmodelElementFields {
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
                },
                direction: value.direction,
                state: value.state,
                message_topic: value.message_topic,
                message_broker: value.message_broker,
                last_update: value.last_update,
                min_interval: value.min_interval,
                max_interval: value.max_interval,
            }
        }
    }
    impl From<BasicEventElementMeta> for BasicEventElementMetaXML {
        fn from(value: BasicEventElementMeta) -> Self {
            Self {
                id_short: value.submodel_element_fields.referable.id_short,
                display_name: value
                    .submodel_element_fields
                    .referable
                    .display_name
                    .map(|values| LangStringTextType { values }),
                description: value
                    .submodel_element_fields
                    .referable
                    .description
                    .map(|values| LangStringTextType { values }),
                #[allow(deprecated)]
                category: value.submodel_element_fields.referable.category,
                extension: value.submodel_element_fields.referable.extensions.extension,
                semantic_id: value.submodel_element_fields.semantics.semantic_id,
                supplemental_semantic_ids: value
                    .submodel_element_fields
                    .semantics
                    .supplemental_semantic_ids,
                qualifiers: value.submodel_element_fields.qualifiable.qualifiers,
                embedded_data_specifications: value
                    .submodel_element_fields
                    .embedded_data_specifications
                    .embedded_data_specifications,
                direction: value.direction,
                state: value.state,
                message_topic: value.message_topic,
                message_broker: value.message_broker,
                last_update: value.last_update,
                min_interval: value.min_interval,
                max_interval: value.max_interval,
            }
        }
    }
}
