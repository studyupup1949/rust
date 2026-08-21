use std::collections::BTreeMap;

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::types::JsonObject;

use super::security::{SecurityRequirement, SecurityScheme};

/// Agent discovery document served from `/.well-known/agent-card.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCard {
    /// Human-readable agent name.
    pub name: String,
    /// Human-readable agent description.
    pub description: String,
    /// Ordered list of supported transport bindings.
    pub supported_interfaces: Vec<AgentInterface>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional agent provider metadata.
    pub provider: Option<AgentProvider>,
    /// Agent implementation version string.
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional human-readable documentation URL.
    pub documentation_url: Option<String>,
    /// Capability flags and advertised extensions.
    pub capabilities: AgentCapabilities,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    /// Named security schemes referenced by requirements.
    pub security_schemes: BTreeMap<String, SecurityScheme>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Security requirements that apply by default.
    pub security_requirements: Vec<SecurityRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Default accepted input modes.
    pub default_input_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Default produced output modes.
    pub default_output_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Skills exposed by the agent.
    pub skills: Vec<AgentSkill>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Optional signatures over the agent card.
    pub signatures: Vec<AgentCardSignature>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional icon URL for UI presentation.
    pub icon_url: Option<String>,
}

/// Transport binding advertised by an agent card.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInterface {
    /// Absolute or relative interface URL.
    pub url: String,
    /// Binding name such as `JSONRPC` or `HTTP+JSON`.
    pub protocol_binding: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant associated with the interface.
    pub tenant: Option<String>,
    /// Protocol version served from the interface.
    pub protocol_version: String,
}

/// Organization metadata for the agent provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProvider {
    /// Provider homepage URL.
    pub url: String,
    /// Provider or organization name.
    pub organization: String,
}

/// Capability flags and extension declarations for an agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Whether streaming operations are supported.
    pub streaming: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Whether push-notification configuration APIs are supported.
    pub push_notifications: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Protocol extensions advertised by the agent.
    pub extensions: Vec<AgentExtension>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Whether `GetExtendedAgentCard` is supported.
    pub extended_agent_card: Option<bool>,
}

/// Extension declaration inside `AgentCapabilities`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentExtension {
    /// Stable extension URI.
    pub uri: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    /// Human-readable extension description.
    pub description: String,
    #[serde(default, skip_serializing_if = "crate::types::is_false")]
    /// Whether the extension is required to interoperate.
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional extension-specific parameters.
    pub params: Option<JsonObject>,
}

/// Skill advertised by an agent card.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkill {
    /// Stable skill identifier.
    pub id: String,
    /// Human-readable skill name.
    pub name: String,
    /// Human-readable skill description.
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Searchable skill tags.
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Example prompts or invocations for the skill.
    pub examples: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Input modes supported by the skill.
    pub input_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Output modes produced by the skill.
    pub output_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Security requirements specific to the skill.
    pub security_requirements: Vec<SecurityRequirement>,
}

/// Signature over an agent card payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCardSignature {
    /// Protected JOSE header segment.
    pub protected: String,
    /// Signature bytes encoded as a string.
    pub signature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional unprotected JOSE header values.
    pub header: Option<JsonObject>,
}

/// Decoded protected header for an agent-card detached JWS signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwsProtectedHeader {
    /// JWS algorithm identifier such as `ES256`.
    pub alg: String,
    /// Key identifier used to resolve the verification key.
    pub kid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional JOSE type, typically `JOSE`.
    pub typ: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional JWK Set URL that can help callers resolve keys.
    pub jku: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    /// Additional JOSE header parameters.
    pub extra: BTreeMap<String, Value>,
}

/// Prepared detached-JWS verification input for an agent-card signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCardSignatureVerificationInput {
    /// Parsed protected JOSE header.
    pub protected_header: JwsProtectedHeader,
    /// Original base64url-encoded protected segment.
    pub protected_segment: String,
    /// Decoded signature bytes.
    pub signature: Vec<u8>,
    /// Detached JWS signing input: `protected + "." + base64url(payload)`.
    pub signing_input: Vec<u8>,
    /// Optional unprotected JOSE header.
    pub unprotected_header: Option<JsonObject>,
}

/// Errors produced by agent-card signature helper APIs.
#[derive(Debug, Error)]
pub enum AgentCardSignatureError {
    /// The agent card does not contain any signatures.
    #[error("agent card does not contain any signatures")]
    MissingSignatures,
    /// The signature protected header could not be base64url-decoded.
    #[error("invalid protected header encoding: {0}")]
    InvalidProtectedEncoding(String),
    /// The signature bytes could not be base64url-decoded.
    #[error("invalid signature encoding: {0}")]
    InvalidSignatureEncoding(String),
    /// The protected header JSON is malformed.
    #[error("invalid protected header JSON: {0}")]
    InvalidProtectedHeader(#[source] serde_json::Error),
    /// No signature used a caller-supported algorithm.
    #[error("no agent-card signature matched the supported algorithms")]
    UnsupportedAlgorithm,
    /// All candidate signatures failed caller-supplied verification.
    #[error("agent-card signature verification failed")]
    VerificationFailed,
    /// JSON serialization needed for canonicalization failed.
    #[error("agent-card serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl AgentCard {
    /// Return a clone of the card with signature blocks removed.
    pub fn unsigned_clone(&self) -> Self {
        let mut card = self.clone();
        card.signatures.clear();
        card
    }

    /// Canonicalize the unsigned agent card for detached-JWS verification.
    pub fn canonical_signing_payload(&self) -> Result<String, AgentCardSignatureError> {
        canonicalize_json(&serde_json::to_value(self.unsigned_clone())?)
    }

    /// Verify any advertised signature using caller-supplied crypto.
    ///
    /// The caller controls key lookup and cryptographic verification. The SDK
    /// prepares detached-JWS inputs and filters signatures by supported
    /// algorithm identifiers.
    pub fn verify_signatures<F>(
        &self,
        supported_algorithms: &[&str],
        mut verifier: F,
    ) -> Result<(), AgentCardSignatureError>
    where
        F: FnMut(&AgentCardSignatureVerificationInput) -> Result<bool, AgentCardSignatureError>,
    {
        if self.signatures.is_empty() {
            return Err(AgentCardSignatureError::MissingSignatures);
        }

        let mut matched_algorithm = false;
        for signature in &self.signatures {
            let input = signature.verification_input(self)?;
            if !supported_algorithms.is_empty()
                && !supported_algorithms.iter().any(|algorithm| {
                    algorithm.eq_ignore_ascii_case(input.protected_header.alg.as_str())
                })
            {
                continue;
            }

            matched_algorithm = true;
            if verifier(&input)? {
                return Ok(());
            }
        }

        if !matched_algorithm {
            return Err(AgentCardSignatureError::UnsupportedAlgorithm);
        }

        Err(AgentCardSignatureError::VerificationFailed)
    }
}

impl AgentCardSignature {
    /// Decode the protected JOSE header from its base64url segment.
    pub fn protected_header(&self) -> Result<JwsProtectedHeader, AgentCardSignatureError> {
        let bytes = base64_url_engine()
            .decode(self.protected.as_bytes())
            .map_err(|error| {
                AgentCardSignatureError::InvalidProtectedEncoding(error.to_string())
            })?;

        serde_json::from_slice(&bytes).map_err(AgentCardSignatureError::InvalidProtectedHeader)
    }

    /// Decode the raw signature bytes from their base64url representation.
    pub fn signature_bytes(&self) -> Result<Vec<u8>, AgentCardSignatureError> {
        base64_url_engine()
            .decode(self.signature.as_bytes())
            .map_err(|error| AgentCardSignatureError::InvalidSignatureEncoding(error.to_string()))
    }

    /// Build the detached-JWS verification input for this signature.
    pub fn verification_input(
        &self,
        card: &AgentCard,
    ) -> Result<AgentCardSignatureVerificationInput, AgentCardSignatureError> {
        let protected_header = self.protected_header()?;
        let signature = self.signature_bytes()?;
        let payload = card.canonical_signing_payload()?;
        let payload_segment = base64_url_engine().encode(payload.as_bytes());
        let signing_input = format!("{}.{}", self.protected, payload_segment).into_bytes();

        Ok(AgentCardSignatureVerificationInput {
            protected_header,
            protected_segment: self.protected.clone(),
            signature,
            signing_input,
            unprotected_header: self.header.clone(),
        })
    }
}

fn canonicalize_json(value: &Value) -> Result<String, AgentCardSignatureError> {
    match value {
        Value::Null => Ok("null".to_owned()),
        Value::Bool(value) => Ok(if *value { "true" } else { "false" }.to_owned()),
        Value::Number(value) => Ok(value.to_string()),
        Value::String(value) => serde_json::to_string(value).map_err(AgentCardSignatureError::from),
        Value::Array(values) => {
            let mut json = String::from("[");
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    json.push(',');
                }
                json.push_str(&canonicalize_json(value)?);
            }
            json.push(']');
            Ok(json)
        }
        Value::Object(values) => {
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();

            let mut json = String::from("{");
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    json.push(',');
                }
                json.push_str(&serde_json::to_string(key)?);
                json.push(':');
                json.push_str(&canonicalize_json(&values[key])?);
            }
            json.push('}');
            Ok(json)
        }
    }
}

fn base64_url_engine() -> &'static base64::engine::GeneralPurpose {
    &base64::engine::general_purpose::URL_SAFE_NO_PAD
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        AgentCapabilities, AgentCard, AgentCardSignature, AgentCardSignatureError, AgentExtension,
        AgentInterface, AgentSkill, JwsProtectedHeader,
    };
    use base64::Engine as _;
    use serde_json::json;

    #[test]
    fn agent_card_round_trip_serialization() {
        let card = AgentCard {
            name: "Echo Agent".to_owned(),
            description: "Replies with the same text".to_owned(),
            supported_interfaces: vec![AgentInterface {
                url: "https://example.com/rpc".to_owned(),
                protocol_binding: "JSONRPC".to_owned(),
                tenant: None,
                protocol_version: "1.0".to_owned(),
            }],
            provider: None,
            version: "0.1.0".to_owned(),
            documentation_url: None,
            capabilities: AgentCapabilities {
                streaming: Some(true),
                push_notifications: Some(false),
                extensions: vec![AgentExtension {
                    uri: "https://example.com/ext/streaming".to_owned(),
                    description: "Streaming support".to_owned(),
                    required: false,
                    params: None,
                }],
                extended_agent_card: Some(false),
            },
            security_schemes: BTreeMap::new(),
            security_requirements: Vec::new(),
            default_input_modes: vec!["text/plain".to_owned()],
            default_output_modes: vec!["text/plain".to_owned()],
            skills: vec![AgentSkill {
                id: "echo".to_owned(),
                name: "Echo".to_owned(),
                description: "Echo back user input".to_owned(),
                tags: vec!["utility".to_owned()],
                examples: vec!["echo hello".to_owned()],
                input_modes: vec!["text/plain".to_owned()],
                output_modes: vec!["text/plain".to_owned()],
                security_requirements: Vec::new(),
            }],
            signatures: Vec::new(),
            icon_url: None,
        };

        let json = serde_json::to_string(&card).expect("card should serialize");
        let round_trip: AgentCard = serde_json::from_str(&json).expect("card should deserialize");

        assert_eq!(round_trip.name, "Echo Agent");
        assert_eq!(
            round_trip.supported_interfaces[0].protocol_binding,
            "JSONRPC"
        );
        assert_eq!(
            round_trip.capabilities.extensions[0].description,
            "Streaming support"
        );
        assert!(!round_trip.capabilities.extensions[0].required);
        assert_eq!(round_trip.skills[0].id, "echo");
    }

    #[test]
    fn signature_helper_decodes_protected_header() {
        let protected = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&json!({
                "alg": "ES256",
                "kid": "key-1",
                "typ": "JOSE",
            }))
            .expect("header should serialize"),
        );
        let signature = AgentCardSignature {
            protected,
            signature: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([1_u8, 2, 3]),
            header: None,
        };

        let header = signature
            .protected_header()
            .expect("protected header should decode");
        assert_eq!(
            header,
            JwsProtectedHeader {
                alg: "ES256".to_owned(),
                kid: "key-1".to_owned(),
                typ: Some("JOSE".to_owned()),
                jku: None,
                extra: BTreeMap::new(),
            }
        );
    }

    #[test]
    fn canonical_signing_payload_omits_signatures() {
        let mut card = sample_card();
        card.signatures.push(sample_signature());

        let payload = card
            .canonical_signing_payload()
            .expect("payload should canonicalize");

        assert!(!payload.contains("\"signatures\""));
        assert!(payload.starts_with("{\"capabilities\""));
    }

    #[test]
    fn verify_signatures_builds_detached_jws_input() {
        let mut card = sample_card();
        let signature = sample_signature();
        let protected = signature.protected.clone();
        card.signatures.push(signature);

        let payload_segment = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
            card.canonical_signing_payload()
                .expect("payload should canonicalize"),
        );
        let expected_input = format!("{protected}.{payload_segment}");

        card.verify_signatures(&["ES256"], |input| {
            assert_eq!(input.protected_header.alg, "ES256");
            assert_eq!(input.protected_header.kid, "key-1");
            assert_eq!(input.signature, vec![1_u8, 2, 3]);
            assert_eq!(input.signing_input, expected_input.as_bytes());
            Ok(true)
        })
        .expect("verification should succeed");
    }

    #[test]
    fn verify_signatures_rejects_cards_without_supported_algorithms() {
        let mut card = sample_card();
        card.signatures.push(sample_signature());

        let error = card
            .verify_signatures(&["RS256"], |_input| Ok(true))
            .expect_err("unsupported algorithms should fail");

        assert!(matches!(
            error,
            AgentCardSignatureError::UnsupportedAlgorithm
        ));
    }

    fn sample_card() -> AgentCard {
        AgentCard {
            name: "Echo Agent".to_owned(),
            description: "Replies with the same text".to_owned(),
            supported_interfaces: vec![AgentInterface {
                url: "https://example.com/rpc".to_owned(),
                protocol_binding: "JSONRPC".to_owned(),
                tenant: None,
                protocol_version: "1.0".to_owned(),
            }],
            provider: None,
            version: "0.1.0".to_owned(),
            documentation_url: None,
            capabilities: AgentCapabilities {
                streaming: Some(true),
                push_notifications: Some(false),
                extensions: vec![AgentExtension {
                    uri: "https://example.com/ext/streaming".to_owned(),
                    description: "Streaming support".to_owned(),
                    required: false,
                    params: None,
                }],
                extended_agent_card: Some(false),
            },
            security_schemes: BTreeMap::new(),
            security_requirements: Vec::new(),
            default_input_modes: vec!["text/plain".to_owned()],
            default_output_modes: vec!["text/plain".to_owned()],
            skills: vec![AgentSkill {
                id: "echo".to_owned(),
                name: "Echo".to_owned(),
                description: "Echo back user input".to_owned(),
                tags: vec!["utility".to_owned()],
                examples: vec!["echo hello".to_owned()],
                input_modes: vec!["text/plain".to_owned()],
                output_modes: vec!["text/plain".to_owned()],
                security_requirements: Vec::new(),
            }],
            signatures: Vec::new(),
            icon_url: None,
        }
    }

    fn sample_signature() -> AgentCardSignature {
        AgentCardSignature {
            protected: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&json!({
                    "alg": "ES256",
                    "kid": "key-1",
                    "typ": "JOSE",
                }))
                .expect("header should serialize"),
            ),
            signature: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([1_u8, 2, 3]),
            header: None,
        }
    }
}
