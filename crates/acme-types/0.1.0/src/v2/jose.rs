#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Defines a JSON web key object.
///
/// For more information, refer to [RFC 8555 § 6.2](https://datatracker.ietf.org/doc/html/rfc8555#section-6.2)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct JsonWebKey {
    #[serde(rename = "kty")]
    pub key_type: String,
    #[serde(rename = "e")]
    pub exponent: String,
    #[serde(rename = "n")]
    pub modulus: String,
}

#[cfg(feature = "json")]
impl JsonWebKey {
    /// Deserializes a JsonWebKey object from a JSON str
    pub fn from_str(s: &str) -> Result<JsonWebKey, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes a JsonWebKey object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Defines a JSON web signature object.
///
/// For more information, refer to [RFC 8555 § 6.2](https://datatracker.ietf.org/doc/html/rfc8555#section-6.2)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct JsonWebSignature {
    pub protected: String,
    pub payload: String,
    pub signature: String,
}

#[cfg(feature = "json")]
impl JsonWebSignature {
    /// Deserializes a JsonWebSignature object from a JSON str
    pub fn from_str(s: &str) -> Result<JsonWebSignature, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes a JsonWebSignature object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Defines the protected data object in the JWS payload.
///
/// For more information, refer to [RFC 8555 § 6.2](https://datatracker.ietf.org/doc/html/rfc8555#section-6.2)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct JsonWebSignatureProtected {
    #[serde(rename = "alg")]
    pub algorithm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "jwk")]
    pub json_web_key: Option<JsonWebKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "kid")]
    pub key_id: Option<String>,
}

#[cfg(feature = "json")]
impl JsonWebSignatureProtected {
    /// Deserializes a JsonWebSignatureProtected object from a JSON str
    pub fn from_str(s: &str) -> Result<JsonWebSignatureProtected, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes a JsonWebSignatureProtected object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}
