#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Defines an ACME directory resource.
///
/// For more information, refer to [RFC 8555 § 7.1.1](https://datatracker.ietf.org/doc/html/rfc8555#section-7.1.1)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Directory {
    /// New nonce URL
    #[cfg_attr(feature = "json", serde(rename = "newNonce"))]
    pub new_nonce: String,
    /// New account URL
    #[cfg_attr(feature = "json", serde(rename = "newAccount"))]
    pub new_account: String,
    /// New order URL
    #[cfg_attr(feature = "json", serde(rename = "newOrder"))]
    pub new_order: String,
    /// New authorization URL
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "newAuthz"))]
    pub new_authorization: Option<String>,
    /// Revoke certificate URL
    #[cfg_attr(feature = "json", serde(rename = "revokeCert"))]
    pub revoke_certificate: String,
    /// Key change URL
    #[cfg_attr(feature = "json", serde(rename = "keyChange"))]
    pub key_change: String,
    /// Metadata object
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "meta"))]
    pub metadata: Option<DirectoryMetadata>,
}

#[cfg(feature = "json")]
impl Directory {
    /// Deserializes a Directory object from a JSON str
    pub fn from_str(s: &str) -> Result<Directory, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes a Directory object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Defines an ACME directory metadata object.
///
/// For more information, refer to [RFC 8555 § 7.1.1](https://datatracker.ietf.org/doc/html/rfc8555#section-7.1.1)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct DirectoryMetadata {
    /// ACME provider Terms of Service URL
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "termsOfService"))]
    pub terms_of_service: Option<String>,
    /// ACME provider website URL
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    pub website: Option<String>,
    /// Array of hostnames recognized for the purpose of CAA record validation
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "caaIdentities"))]
    pub caa_identities: Option<Vec<String>>,
    /// Whether or not an external account is required for account registration by the ACME provider
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "externalAccountRequired"))]
    pub external_account_required: Option<bool>,
}
