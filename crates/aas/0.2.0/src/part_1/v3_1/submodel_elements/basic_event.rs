use crate::part_1::v3_1::primitives::{DateTimeUTC, MessageTopic};
use crate::part_1::v3_1::reference::Reference;
use crate::part_1::v3_1::submodel_elements::SubmodelElementFields;
use crate::part_1::{MetamodelError, ToJsonMetamodel};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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
            .map_err(|e| MetamodelError::FailedSerialisation(e))
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
