#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Defines an ACME account registration object.
///
/// For more information, refer to [RFC 8555 § 7.3](https://datatracker.ietf.org/doc/html/rfc8555#section-7.3)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct NewAccount {
    /// Array of URLs that can be used by the ACME provider to contact the client
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    pub contact: Option<String>,
    /// Confirmation client has agreed to the ACME provider's Terms of Service
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "termsOfServiceAgreed"))]
    pub terms_of_service_agreed: Option<bool>,
    /// Prevent account creation if one does not exist
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "onlyReturnExisting"))]
    pub only_return_existing: Option<bool>,
    /// External account object
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "externalAccountBinding"))]
    pub external_account_binding: Option<super::JsonWebSignature>,
}

#[cfg(feature = "json")]
impl NewAccount {
    /// Deserializes a NewAccount object from a JSON str
    pub fn from_str(s: &str) -> Result<NewAccount, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes a NewAccount object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Defines an ACME account update object.
///
/// For more information, refer to [RFC 8555 § 7.3.2](https://datatracker.ietf.org/doc/html/rfc8555#section-7.3.2)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct AccountUpdate {
    /// This field should not be set unless updating the ACME client's contact details.
    ///
    /// For more information, refer to [RFC 8555 § 7.3.2](https://datatracker.ietf.org/doc/html/rfc8555#section-7.3.2)
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    pub contact: Option<Vec<String>>,
    /// This field should not be set unless deactivating the ACME client.
    ///
    /// For more information, refer to [RFC 8555 § 7.3.6](https://datatracker.ietf.org/doc/html/rfc8555#section-7.3.6)
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    pub status: Option<AccountStatus>,
    /// This field should not be set unless the client is agreeing to the current ACME provider's Terms of Service.
    ///
    /// For more information, refer to [RFC 8555 § 7.3.3](https://datatracker.ietf.org/doc/html/rfc8555#section-7.3.3)
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "termsOfServiceAgreed"))]
    pub terms_of_service_agreed: Option<bool>,
}

#[cfg(feature = "json")]
impl AccountUpdate {
    /// Deserializes an AccountUpdate object from a JSON str
    pub fn from_str(s: &str) -> Result<AccountUpdate, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes an AccountUpdate object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Defines an ACME account resource.
///
/// For more information, refer to [RFC 8555 § 7.1.2](https://datatracker.ietf.org/doc/html/rfc8555#section-7.1.2)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Account {
    /// ACME account status
    pub status: AccountStatus,
    /// Array of URLs that can be used by the ACME provider to contact the client
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    pub contact: Option<Vec<String>>,
    /// If set to true, indicates that the ACME account has agreed to the ACME provider's Terms of Service
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "termsOfServiceAgreed"))]
    pub terms_of_service_agreed: Option<bool>,
    /// External account object
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "json", serde(rename = "externalAccountBinding"))]
    pub external_account_binding: Option<super::JsonWebSignature>,
    /// URL from which a list of orders submitted by the ACME account can be retrieved.
    ///
    /// For more information, refer to [RFC 8555 § 7.1.2.1](https://datatracker.ietf.org/doc/html/rfc8555#section-7.1.2.1)
    pub orders: String,
}

#[cfg(feature = "json")]
impl Account {
    /// Deserializes an Account object from a JSON str
    pub fn from_str(s: &str) -> Result<Account, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes an Account object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Defines an ACME account orders object.
///
/// For more information, refer to [RFC 8555 § 7.1.2.1](https://datatracker.ietf.org/doc/html/rfc8555#section-7.1.2.1)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct AccountOrders {
    /// Array of URLs identifying orders belonging to the ACME account
    pub orders: Vec<String>,
}

#[cfg(feature = "json")]
impl AccountOrders {
    /// Deserializes an AccountOrders object from a JSON str
    pub fn from_str(s: &str) -> Result<AccountOrders, serde_json::error::Error> {
        serde_json::from_str(s)
    }

    /// Serializes an AccountOrders object to a JSON String
    pub fn to_string(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string(self)
    }
}

/// Account resource status values
///
/// For more information, refer to [RFC 8555 § 7.1.6](https://datatracker.ietf.org/doc/html/rfc8555#section-7.1.6)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum AccountStatus {
    #[cfg_attr(feature = "json", serde(rename = "valid"))]
    Valid,
    #[cfg_attr(feature = "json", serde(rename = "deactivated"))]
    Deactivated,
    #[cfg_attr(feature = "json", serde(rename = "revoked"))]
    Revoked,
}
