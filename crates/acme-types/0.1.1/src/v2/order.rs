#[cfg(feature = "json")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Defines a new ACME order object
///
/// For more information, refer to [RFC 8555 § 7.4](https://datatracker.ietf.org/doc/html/rfc8555#section-7.4)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct NewOrder {
    /// Array of requested identifiers
    pub identifiers: Vec<super::Identifier>,
    /// Requested value for certificate's notBefore value
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "notBefore"))]
    pub not_before: Option<String>,
    /// Requested value for certificate's notAfter value
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "notAfter"))]
    pub not_after: Option<String>,
}

#[cfg(feature = "json")]
impl NewOrder {
    /// Deserializes a NewOrder object from a JSON str
    pub fn from_str(s: &str) -> Result<NewOrder, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes a NewOrder object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Defines a new ACME order resource
///
/// For more information, refer to [RFC 8555 § 9.7.2](https://datatracker.ietf.org/doc/html/rfc8555#section-9.7.2)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Order {
    /// Order status
    pub status: OrderStatus,
    /// Order expiration time
    pub expires: String,
    /// Array of requested identifiers
    pub identifiers: Vec<super::Identifier>,
    /// Requested value for certificate's notBefore value
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "notBefore"))]
    pub not_before: Option<String>,
    /// Requested value for certificate's notAfter value
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "notAfter"))]
    pub not_after: Option<String>,
    /// Error encountered during domain validation, certificate issuance, etc.
    pub error: Option<String>,
    /// Authorizations which need to be completed in order to finalize the order
    pub authorizations: Option<String>,
    /// URL to finalize order
    pub finalize: String,
    /// URL to retrieve certificate issued by ACME provider
    pub certificate: String,
}

#[cfg(feature = "json")]
impl Order {
    /// Deserializes an Order object from a JSON str
    pub fn from_str(s: &str) -> Result<Order, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes an Order object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Defines an ACME order finalize object
///
/// For more information, refer to [RFC 8555 § 7.4](https://datatracker.ietf.org/doc/html/rfc8555#section-7.4)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct OrderFinalize {
    /// CSR for the requested certificate (base64url-encoding of the DER-encoded CSR)
    #[cfg_attr(feature = "json", serde(rename = "csr"))]
    pub certificate_signing_request: String,
}

#[cfg(feature = "json")]
impl OrderFinalize {
    /// Deserializes an OrderFinalize object from a JSON str
    pub fn from_str(s: &str) -> Result<OrderFinalize, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes an OrderFinalize object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Order resource status values
///
/// For more information, refer to [RFC 8555 § 7.1.6](https://datatracker.ietf.org/doc/html/rfc8555#section-7.1.6)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum OrderStatus {
    #[cfg_attr(feature = "json", serde(rename = "pending"))]
    Pending,
    #[cfg_attr(feature = "json", serde(rename = "ready"))]
    Ready,
    #[cfg_attr(feature = "json", serde(rename = "processing"))]
    Processing,
    #[cfg_attr(feature = "json", serde(rename = "valid"))]
    Valid,
    #[cfg_attr(feature = "json", serde(rename = "invalid"))]
    Invalid,
}

/// Defines a certificate revocation request
///
/// For more information, refer to [RFC 8555 § 7.6](https://datatracker.ietf.org/doc/html/rfc8555#section-7.6)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct CertificateRevocation {
    /// base64url-encoding of the DER-encoded certificate to revoke
    pub certificate: String,
    /// Reason for certificate revocation
    ///
    /// For more information, refer to [RFC 5280 § 5.3.1](https://datatracker.ietf.org/doc/html/rfc5280#section-5.3.1)
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        feature = "json",
        serde(serialize_with = "certificate_revocation_reason_serialize")
    )]
    #[cfg_attr(
        feature = "json",
        serde(deserialize_with = "certificate_revocation_reason_deserialize")
    )]
    pub reason: Option<CertificateRevocationReason>,
}

#[cfg(feature = "json")]
impl CertificateRevocation {
    /// Deserializes a CertificateRevocation object from a JSON str
    pub fn from_str(s: &str) -> Result<CertificateRevocation, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes a CertificateRevocation object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Certificate revocation reason values
///
/// For more information, refer to [RFC 5280 § 5.3.1](https://datatracker.ietf.org/doc/html/rfc5280#section-5.3.1)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum CertificateRevocationReason {
    Unspecified,
    KeyCompromise,
    CertificateAuthorityCompromise,
    AffiliationChanged,
    Superseded,
    CessationOfOperation,
    CertificateHold,
    RemoveFromCertificateRevocationList,
    PrivilegeWithdrawn,
    AuthorityAttributeCompromise,
    Other(i32),
}

#[cfg(feature = "json")]
fn certificate_revocation_reason_deserialize<'de, D>(
    deserializer: D,
) -> Result<Option<CertificateRevocationReason>, D::Error>
where
    D: Deserializer<'de>,
{
    use self::CertificateRevocationReason::*;

    let n = String::deserialize(deserializer)?.parse::<i32>().unwrap();

    Ok(Some(match n {
        0 => Unspecified,
        1 => KeyCompromise,
        2 => CertificateAuthorityCompromise,
        3 => AffiliationChanged,
        4 => Superseded,
        5 => CessationOfOperation,
        6 => CertificateHold,
        8 => RemoveFromCertificateRevocationList,
        9 => PrivilegeWithdrawn,
        10 => AuthorityAttributeCompromise,
        _ => Other(n),
    }))
}

#[cfg(feature = "json")]
fn certificate_revocation_reason_serialize<S>(
    type_: &Option<CertificateRevocationReason>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use self::CertificateRevocationReason::*;

    serializer.serialize_i32(match type_.clone().unwrap() {
        Other(i) => i,
        Unspecified => 0,
        KeyCompromise => 1,
        CertificateAuthorityCompromise => 2,
        AffiliationChanged => 3,
        Superseded => 4,
        CessationOfOperation => 5,
        CertificateHold => 6,
        RemoveFromCertificateRevocationList => 8,
        PrivilegeWithdrawn => 9,
        AuthorityAttributeCompromise => 10,
    })
}
