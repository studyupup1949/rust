use crate::part_1::v3_1::primitives::{DateTimeUTC, MessageTopicType};
use crate::part_1::v3_1::reference::Reference;
use crate::part_1::v3_1::submodel_elements::SubmodelElementFields;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct BasicEventElement {
    #[serde(flatten)]
    submodel_element_fields: SubmodelElementFields,

    pub observed: Reference,

    pub direction: Direction,

    pub state: StateOfEvent,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "messageTopic")]
    pub message_topic: Option<MessageTopicType>,

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

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Display, EnumString)]
pub enum StateOfEvent {
    #[serde(rename = "on")]
    On,
    #[serde(rename = "off")]
    Off,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Display, EnumString)]
pub enum Direction {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "output")]
    Output,
}
