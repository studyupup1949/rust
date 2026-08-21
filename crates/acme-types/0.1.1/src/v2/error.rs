#[cfg(feature = "json")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Defines an ACME error object
///
/// For more information, refer to [RFC 8555 § 6.7](https://datatracker.ietf.org/doc/html/rfc8555#section-6.7)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Error {
    /// Error type
    #[cfg_attr(feature = "json", serde(rename = "type"))]
    #[cfg_attr(feature = "json", serde(serialize_with = "error_type_serialize"))]
    #[cfg_attr(feature = "json", serde(deserialize_with = "error_type_deserialize"))]
    pub type_: ErrorType,
    /// Error description
    pub detail: String,
    /// Optional sub-problem documents
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "subproblems"))]
    pub sub_problems: Option<Vec<ProblemDocument>>,
}

#[cfg(feature = "json")]
impl Error {
    /// Deserializes an Error object from a JSON str
    pub fn from_str(s: &str) -> Result<Error, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes an Error object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Defines an ACME error problem document object
///
/// For more information, refer to [RFC 8555 § 6.7.1](https://datatracker.ietf.org/doc/html/rfc8555#section-6.7.1)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct ProblemDocument {
    /// Error type
    #[cfg_attr(feature = "json", serde(rename = "type"))]
    #[cfg_attr(feature = "json", serde(serialize_with = "error_type_serialize"))]
    #[cfg_attr(feature = "json", serde(deserialize_with = "error_type_deserialize"))]
    pub type_: ErrorType,
    /// Error description
    pub detail: String,
    /// Optional order identifier associated with the error
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    pub identifier: Option<super::Identifier>,
}

/// Non-exhaustive list of ACME error types
///
/// For more information, refer to [RFC 8555 § 6.7](https://datatracker.ietf.org/doc/html/rfc8555#section-6.7)
#[derive(Clone, Debug)]
pub enum ErrorType {
    AccountDoesNotExist,
    AlreadyRevoked,
    BadCertificateSigningRequest,
    BadNonce,
    BadPublicKey,
    BadRevocationReason,
    BadSignatureAlgorithm,
    CertificationAuthorityAuthorization,
    Compound,
    Connection,
    Dns,
    ExternalAccountRequired,
    IncorrectResponse,
    InvalidContact,
    Malformed,
    OrderNotReady,
    RateLimited,
    RejectedIdentifier,
    ServerInternal,
    Tls,
    Unauthorized,
    UnsupportedContact,
    UnsupportedIdentifier,
    UserActionRequired,
    Other(String),
}

#[cfg(feature = "json")]
fn error_type_deserialize<'de, D>(deserializer: D) -> Result<ErrorType, D::Error>
where
    D: Deserializer<'de>,
{
    use self::ErrorType::*;
    let s = String::deserialize(deserializer)?;

    match s.starts_with("urn:ietf:params:acme:error:") {
        true => match s.strip_prefix("urn:ietf:params:acme:error:").unwrap() {
            "accountDoesNotExist" => Ok(AccountDoesNotExist),
            "alreadyRevoked" => Ok(AccountDoesNotExist),
            "badCSR" => Ok(AccountDoesNotExist),
            "badNonce" => Ok(AccountDoesNotExist),
            "badPublicKey" => Ok(AccountDoesNotExist),
            "badRevocationReason" => Ok(AccountDoesNotExist),
            "badSignatureAlgorithm" => Ok(AccountDoesNotExist),
            "caa" => Ok(AccountDoesNotExist),
            "compound" => Ok(AccountDoesNotExist),
            "connection" => Ok(AccountDoesNotExist),
            "dns" => Ok(AccountDoesNotExist),
            "externalAccountRequired" => Ok(AccountDoesNotExist),
            "incorrectResponse" => Ok(AccountDoesNotExist),
            "invalidContact" => Ok(AccountDoesNotExist),
            "malformed" => Ok(AccountDoesNotExist),
            "orderNotReady" => Ok(AccountDoesNotExist),
            "rateLimited" => Ok(AccountDoesNotExist),
            "rejectedIdentifier" => Ok(AccountDoesNotExist),
            "serverInternal" => Ok(AccountDoesNotExist),
            "tls" => Ok(AccountDoesNotExist),
            "unauthorized" => Ok(AccountDoesNotExist),
            "unsupportedContact" => Ok(AccountDoesNotExist),
            "unsupportedIdentifier" => Ok(AccountDoesNotExist),
            "userActionRequired" => Ok(AccountDoesNotExist),
            _ => Ok(Other(s)),
        },
        false => Ok(Other(s)),
    }
}

#[cfg(feature = "json")]
fn error_type_serialize<S>(type_: &ErrorType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use self::ErrorType::*;

    serializer.serialize_str(&match type_ {
        Other(s) => s.to_string(),
        _ => {
            format!(
                "urn:ietf:params:acme:error:{}",
                match type_ {
                    AccountDoesNotExist => "accountDoesNotExist",
                    AlreadyRevoked => "alreadyRevoked",
                    BadCertificateSigningRequest => "badCSR",
                    BadNonce => "badNonce",
                    BadPublicKey => "badPublicKey",
                    BadRevocationReason => "badRevocationReason",
                    BadSignatureAlgorithm => "badSignatureAlgorithm",
                    CertificationAuthorityAuthorization => "caa",
                    Compound => "compound",
                    Connection => "connection",
                    Dns => "dns",
                    ExternalAccountRequired => "externalAccountRequired",
                    IncorrectResponse => "incorrectResponse",
                    InvalidContact => "invalidContact",
                    Malformed => "malformed",
                    OrderNotReady => "orderNotReady",
                    RateLimited => "rateLimited",
                    RejectedIdentifier => "rejectedIdentifier",
                    ServerInternal => "serverInternal",
                    Tls => "tls",
                    Unauthorized => "unauthorized",
                    UnsupportedContact => "unsupportedContact",
                    UnsupportedIdentifier => "unsupportedIdentifier",
                    UserActionRequired => "userActionRequired",
                    _ => {
                        panic!("I've truly fucked up")
                    }
                }
            )
        }
    })
}
