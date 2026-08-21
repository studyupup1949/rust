use thiserror::Error;

/// Errors produced when parsing an address.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum ParseError {
    /// Input string was empty or whitespace
    #[error("input is empty")]
    Empty,

    /// No host component found
    #[error("missing host")]
    MissingHost,

    /// Scheme is not valid
    #[error("invalid scheme: {0}")]
    InvalidScheme(String),

    /// Host failed to parse
    #[error("invalid host: {0}")]
    InvalidHost(#[from] HostError),

    /// Port is not a valid number
    #[error("invalid port: {0}")]
    InvalidPort(String),

    /// General parse failure
    #[error("invalid address: {0}")]
    Invalid(String),
}

/// Errors specific to host parsing.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum HostError {
    /// Host string was empty
    #[error("empty host")]
    Empty,
    /// IPv6 literal could not be parsed
    #[error("malformed IPv6 literal: {0}")]
    BadIpv6(String),
    /// Domain name is malformed
    #[error("malformed domain: {0}")]
    BadDomain(String),

    #[cfg(feature = "idna")]
    /// idna conversion errors
    #[error("error converting domain '{0}'")]
    ConversionError(String),
}
