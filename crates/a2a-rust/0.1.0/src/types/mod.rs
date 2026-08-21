//! Core A2A protocol types.

use base64::Engine as _;
use serde::{Deserialize, Deserializer, Serializer};

/// Agent discovery and capability types.
pub mod agent_card;
/// Validated helper type for application-level agent identifiers.
pub mod agent_id;
/// Auth-required metadata helpers and conventions.
pub mod auth;
/// Messages, parts, and artifacts exchanged by agents.
pub mod message;
/// Push-notification configuration types.
pub mod push;
/// Operation request payloads.
pub mod requests;
/// Operation response payloads and stream events.
pub mod responses;
/// Security scheme and requirement types.
pub mod security;
/// Task state and status models.
pub mod task;

pub use self::agent_card::*;
pub use self::agent_id::*;
pub use self::auth::*;
pub use self::message::*;
pub use self::push::*;
pub use self::requests::*;
pub use self::responses::*;
pub use self::security::*;
pub use self::task::*;

/// Shared JSON object alias used across metadata-bearing protocol types.
pub type JsonObject = serde_json::Map<String, serde_json::Value>;

pub(crate) fn is_false(value: &bool) -> bool {
    !*value
}

pub(crate) mod base64_bytes {
    use super::*;

    const ENGINE: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

    pub(crate) mod option {
        use super::*;

        pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match value {
                Some(bytes) => serializer.serialize_some(&ENGINE.encode(bytes)),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let encoded = Option::<String>::deserialize(deserializer)?;
            encoded
                .map(|encoded| ENGINE.decode(encoded).map_err(serde::de::Error::custom))
                .transpose()
        }
    }
}
