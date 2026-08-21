use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(tag = "type", content = "value")]
pub enum Key {
    Variant(u8, u8),
    Custom(CustomKey),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomKey {
    pub name: String,
    #[serde(flatten)]
    pub value: CustomValue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bytes32(pub [u8; 32]);

impl Serialize for Bytes32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(self.0)))
    }
}

impl<'de> Deserialize<'de> for Bytes32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let hex_part = s.strip_prefix("0x").unwrap_or(&s);
        let bytes = hex::decode(hex_part).map_err(serde::de::Error::custom)?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 32 bytes"))?;
        Ok(Self(arr))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct U64Text(pub u64);

impl Serialize for U64Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for U64Text {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = U64Text;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a u64 string or integer")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(U64Text(value))
            }

            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(U64Text(u64::try_from(value).map_err(E::custom)?))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(U64Text(value.parse::<u64>().map_err(E::custom)?))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct U128Text(pub u128);

impl Serialize for U128Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for U128Text {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = U128Text;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a u128 string or integer")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(U128Text(value.into()))
            }

            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(U128Text(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(U128Text(value.parse::<u128>().map_err(E::custom)?))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CustomValue {
    Bytes32(Bytes32),
    U32(u32),
    U64(U64Text),
    U128(U128Text),
    String(String),
    Bool(bool),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EventRef {
    pub block_number: u32,
    pub event_index: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DecodedEvent {
    pub block_number: u32,
    pub event_index: u16,
    pub event: StoredEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StoredEvent {
    pub spec_version: u32,
    pub pallet_name: String,
    pub event_name: String,
    pub pallet_index: u8,
    pub variant_index: u8,
    pub event_index: u16,
    pub fields: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EventMeta {
    pub index: u8,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PalletMeta {
    pub index: u8,
    pub name: String,
    pub events: Vec<EventMeta>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventsResponse {
    pub key: Key,
    pub events: Vec<EventRef>,
    #[serde(default)]
    pub decoded_events: Vec<DecodedEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventMatch {
    pub event_ref: EventRef,
    pub decoded_event: Option<DecodedEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SubscriptionTarget {
    Status,
    Events { key: Key },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusUpdate {
    pub spans: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventNotification {
    pub key: Key,
    pub event: EventRef,
    pub decoded_event: Option<DecodedEvent>,
}

impl StoredEvent {
    pub fn field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    pub fn variant(&self) -> (u8, u8) {
        (self.pallet_index, self.variant_index)
    }
}

impl DecodedEvent {
    pub fn pallet_name(&self) -> &str {
        &self.event.pallet_name
    }

    pub fn event_name(&self) -> &str {
        &self.event.event_name
    }

    pub fn variant(&self) -> (u8, u8) {
        self.event.variant()
    }

    pub fn field(&self, name: &str) -> Option<&Value> {
        self.event.field(name)
    }
}

impl EventsResponse {
    pub fn event_matches(&self) -> Vec<EventMatch> {
        let decoded_by_ref: HashMap<(u32, u16), DecodedEvent> = self
            .decoded_events
            .iter()
            .cloned()
            .map(|decoded| ((decoded.block_number, decoded.event_index), decoded))
            .collect();

        self.events
            .iter()
            .cloned()
            .map(|event_ref| EventMatch {
                decoded_event: decoded_by_ref
                    .get(&(event_ref.block_number, event_ref.event_index))
                    .cloned(),
                event_ref,
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RequestMessage<T> {
    pub id: u64,
    #[serde(rename = "type")]
    pub message_type: &'static str,
    #[serde(flatten)]
    pub payload: T,
}

#[derive(Serialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct EmptyPayload {}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct GetEventsPayload {
    pub key: Key,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<EventRef>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SubscribeEventsPayload {
    pub key: Key,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Envelope {
    pub id: Option<u64>,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(default)]
    pub data: Option<Value>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionStatusPayload {
    pub action: String,
    pub target: SubscriptionTarget,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionTerminatedPayload {
    pub reason: String,
    pub message: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventNotificationPayload {
    pub key: Key,
    pub event: EventRef,
    pub decoded_event: Option<DecodedEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::value::{
        Error as ValueError, StrDeserializer, U128Deserializer, U64Deserializer,
    };
    use serde::Deserialize;
    use serde_json::json;

    fn bytes32_hex(byte: u8) -> String {
        format!("0x{}", hex::encode([byte; 32]))
    }

    #[test]
    fn bytes32_serializes_as_prefixed_hex() {
        assert_eq!(
            serde_json::to_string(&Bytes32([0xAB; 32])).unwrap(),
            format!("\"{}\"", bytes32_hex(0xAB))
        );
    }

    #[test]
    fn bytes32_deserializes_with_and_without_prefix() {
        let prefixed =
            serde_json::from_str::<Bytes32>(&format!("\"{}\"", bytes32_hex(0x11))).unwrap();
        let unprefixed =
            serde_json::from_str::<Bytes32>(&format!("\"{}\"", "22".repeat(32))).unwrap();

        assert_eq!(prefixed, Bytes32([0x11; 32]));
        assert_eq!(unprefixed, Bytes32([0x22; 32]));
    }

    #[test]
    fn bytes32_rejects_wrong_length() {
        let error = serde_json::from_str::<Bytes32>("\"0x1234\"")
            .unwrap_err()
            .to_string();
        assert!(error.contains("expected 32 bytes"));
    }

    #[test]
    fn u64_text_serializes_and_deserializes_multiple_input_shapes() {
        assert_eq!(serde_json::to_string(&U64Text(42)).unwrap(), "\"42\"");
        assert_eq!(
            U64Text::deserialize(U64Deserializer::<ValueError>::new(42)).unwrap(),
            U64Text(42)
        );
        assert_eq!(
            U64Text::deserialize(U128Deserializer::<ValueError>::new(42)).unwrap(),
            U64Text(42)
        );
        assert_eq!(
            U64Text::deserialize(StrDeserializer::<ValueError>::new("42")).unwrap(),
            U64Text(42)
        );
    }

    #[test]
    fn u128_text_serializes_and_deserializes_multiple_input_shapes() {
        assert_eq!(serde_json::to_string(&U128Text(42)).unwrap(), "\"42\"");
        assert_eq!(
            U128Text::deserialize(U64Deserializer::<ValueError>::new(42)).unwrap(),
            U128Text(42)
        );
        assert_eq!(
            U128Text::deserialize(U128Deserializer::<ValueError>::new(42)).unwrap(),
            U128Text(42)
        );
        assert_eq!(
            U128Text::deserialize(StrDeserializer::<ValueError>::new("42")).unwrap(),
            U128Text(42)
        );
    }

    #[test]
    fn custom_value_serializes_all_scalar_kinds() {
        let cases = [
            (
                CustomValue::Bytes32(Bytes32([0xAB; 32])),
                json!({"kind": "bytes32", "value": bytes32_hex(0xAB)}),
            ),
            (CustomValue::U32(7), json!({"kind": "u32", "value": 7})),
            (
                CustomValue::U64(U64Text(8)),
                json!({"kind": "u64", "value": "8"}),
            ),
            (
                CustomValue::U128(U128Text(9)),
                json!({"kind": "u128", "value": "9"}),
            ),
            (
                CustomValue::String("slug".into()),
                json!({"kind": "string", "value": "slug"}),
            ),
            (
                CustomValue::Bool(true),
                json!({"kind": "bool", "value": true}),
            ),
        ];

        for (value, expected) in cases {
            assert_eq!(serde_json::to_value(value).unwrap(), expected);
        }
    }

    #[test]
    fn key_deserializes_variant_and_custom_shapes() {
        let variant =
            serde_json::from_value::<Key>(json!({"type": "Variant", "value": [5, 3]})).unwrap();
        let custom = serde_json::from_value::<Key>(json!({
            "type": "Custom",
            "value": {"name": "published", "kind": "bool", "value": true}
        }))
        .unwrap();

        assert_eq!(variant, Key::Variant(5, 3));
        assert_eq!(
            custom,
            Key::Custom(CustomKey {
                name: "published".into(),
                value: CustomValue::Bool(true),
            })
        );
    }

    #[test]
    fn serializes_empty_payload_request_without_extra_fields() {
        let request = RequestMessage {
            id: 1,
            message_type: "Status",
            payload: EmptyPayload::default(),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json, json!({"id": 1, "type": "Status"}));
    }

    #[test]
    fn serializes_get_events_request_in_server_shape() {
        let request = RequestMessage {
            id: 3,
            message_type: "GetEvents",
            payload: GetEventsPayload {
                key: Key::Custom(CustomKey {
                    name: "item_id".into(),
                    value: CustomValue::Bytes32(Bytes32([0x12; 32])),
                }),
                limit: Some(25),
                before: Some(EventRef {
                    block_number: 50,
                    event_index: 3,
                }),
            },
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["id"], 3);
        assert_eq!(json["type"], "GetEvents");
        assert_eq!(json["key"]["type"], "Custom");
        assert_eq!(json["key"]["value"]["name"], "item_id");
        assert_eq!(json["key"]["value"]["kind"], "bytes32");
        assert_eq!(json["limit"], 25);
        assert_eq!(json["before"]["blockNumber"], 50);
        assert_eq!(json["before"]["eventIndex"], 3);
    }

    #[test]
    fn serializes_get_events_request_without_optional_fields() {
        let request = RequestMessage {
            id: 4,
            message_type: "GetEvents",
            payload: GetEventsPayload {
                key: Key::Custom(CustomKey {
                    name: "ref_index".into(),
                    value: CustomValue::U32(42),
                }),
                limit: None,
                before: None,
            },
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            json!({
                "id": 4,
                "type": "GetEvents",
                "key": {"type": "Custom", "value": {"name": "ref_index", "kind": "u32", "value": 42}}
            })
        );
    }

    #[test]
    fn serializes_subscribe_events_request_in_server_shape() {
        let request = RequestMessage {
            id: 7,
            message_type: "SubscribeEvents",
            payload: SubscribeEventsPayload {
                key: Key::Custom(CustomKey {
                    name: "account_id".into(),
                    value: CustomValue::Bytes32(Bytes32([0xAB; 32])),
                }),
            },
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["type"], "SubscribeEvents");
        assert_eq!(json["key"]["value"]["kind"], "bytes32");
    }

    #[test]
    fn deserializes_events_response_payload() {
        let payload = serde_json::from_value::<EventsResponse>(json!({
            "key": {"type": "Custom", "value": {"name": "ref_index", "kind": "u32", "value": 42}},
            "events": [{"blockNumber": 50, "eventIndex": 3}],
            "decodedEvents": [{
                "blockNumber": 50,
                "eventIndex": 3,
                "event": {
                    "specVersion": 1234,
                    "palletName": "Referenda",
                    "eventName": "Submitted",
                    "palletIndex": 42,
                    "variantIndex": 0,
                    "eventIndex": 3,
                    "fields": {"index": 42}
                }
            }]
        }))
        .unwrap();

        assert_eq!(payload.events.len(), 1);
        assert_eq!(payload.decoded_events.len(), 1);
    }

    #[test]
    fn deserializes_events_response_without_decoded_events() {
        let payload = serde_json::from_value::<EventsResponse>(json!({
            "key": {"type": "Custom", "value": {"name": "ref_index", "kind": "u32", "value": 42}},
            "events": [{"blockNumber": 50, "eventIndex": 3}]
        }))
        .unwrap();

        assert_eq!(payload.events.len(), 1);
        assert!(payload.decoded_events.is_empty());
    }

    #[test]
    fn stored_event_and_events_response_helpers_work() {
        let payload = serde_json::from_value::<EventsResponse>(json!({
            "key": {"type": "Custom", "value": {"name": "ref_index", "kind": "u32", "value": 42}},
            "events": [
                {"blockNumber": 50, "eventIndex": 3},
                {"blockNumber": 49, "eventIndex": 1}
            ],
            "decodedEvents": [{
                "blockNumber": 50,
                "eventIndex": 3,
                "event": {
                    "specVersion": 1234,
                    "palletName": "Referenda",
                    "eventName": "Submitted",
                    "palletIndex": 42,
                    "variantIndex": 0,
                    "eventIndex": 3,
                    "fields": {"index": 42}
                }
            }]
        }))
        .unwrap();

        assert_eq!(payload.decoded_events[0].pallet_name(), "Referenda");
        assert_eq!(payload.decoded_events[0].event_name(), "Submitted");
        assert_eq!(payload.decoded_events[0].variant(), (42, 0));
        assert_eq!(payload.decoded_events[0].field("index"), Some(&json!(42)));

        let matches = payload.event_matches();
        assert_eq!(matches.len(), 2);
        assert!(matches[0].decoded_event.is_some());
        assert!(matches[1].decoded_event.is_none());
    }

    #[test]
    fn stored_event_field_returns_none_for_non_object_fields() {
        let event = StoredEvent {
            spec_version: 1,
            pallet_name: "Example".into(),
            event_name: "Positional".into(),
            pallet_index: 1,
            variant_index: 2,
            event_index: 3,
            fields: json!([1, 2, 3]),
        };

        assert_eq!(event.variant(), (1, 2));
        assert!(event.field("0").is_none());
    }

    #[test]
    fn deserializes_subscription_status_payload() {
        let payload = serde_json::from_value::<SubscriptionStatusPayload>(json!({
            "action": "subscribed",
            "target": {"type": "events", "key": {"type": "Custom", "value": {"name": "ref_index", "kind": "u32", "value": 42}}}
        }))
        .unwrap();

        assert_eq!(payload.action, "subscribed");
        assert_eq!(
            payload.target,
            SubscriptionTarget::Events {
                key: Key::Custom(CustomKey {
                    name: "ref_index".into(),
                    value: CustomValue::U32(42),
                })
            }
        );
    }

    #[test]
    fn deserializes_subscription_terminated_payload() {
        let payload = serde_json::from_value::<SubscriptionTerminatedPayload>(json!({
            "reason": "backpressure",
            "message": "subscriber disconnected due to backpressure"
        }))
        .unwrap();

        assert_eq!(payload.reason, "backpressure");
        assert_eq!(
            payload.message,
            "subscriber disconnected due to backpressure"
        );
    }

    #[test]
    fn deserializes_event_notification_payload() {
        let payload = serde_json::from_value::<EventNotificationPayload>(json!({
            "key": {"type": "Custom", "value": {"name": "item_id", "kind": "bytes32", "value": bytes32_hex(0x11)}},
            "event": {"blockNumber": 50, "eventIndex": 3},
            "decodedEvent": {
                "blockNumber": 50,
                "eventIndex": 3,
                "event": {
                    "specVersion": 1234,
                    "palletName": "Content",
                    "eventName": "PublishRevision",
                    "palletIndex": 42,
                    "variantIndex": 1,
                    "eventIndex": 3,
                    "fields": {"revision_id": 1}
                }
            }
        }))
        .unwrap();

        assert_eq!(payload.event.block_number, 50);
        assert!(payload.decoded_event.is_some());
    }

    #[test]
    fn envelope_defaults_missing_data_to_none() {
        let envelope = serde_json::from_value::<Envelope>(json!({
            "id": 1,
            "type": "subscriptionStatus"
        }))
        .unwrap();

        assert_eq!(envelope.id, Some(1));
        assert_eq!(envelope.message_type, "subscriptionStatus");
        assert_eq!(envelope.data, None);
    }
}
