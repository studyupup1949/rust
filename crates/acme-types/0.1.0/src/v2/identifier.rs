#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Defines the identifier object in the Order and NewAuthorization resources
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Identifier {
    /// Identifier type
    #[cfg_attr(feature = "json", serde(rename = "type"))]
    pub type_: IdentifierType,
    /// Identifier value
    pub value: String,
}

/// Order and authorization identifier type values
#[derive(Clone, Debug)]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub enum IdentifierType {
    #[cfg_attr(feature = "json", serde(rename = "dns"))]
    Dns,
}
