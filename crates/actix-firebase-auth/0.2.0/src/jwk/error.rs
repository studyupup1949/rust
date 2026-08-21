use thiserror::Error;

pub(super) type VerificationResult<T> = std::result::Result<T, VerificationError>;

/// Errors that can occur during token verification.
#[derive(Debug, Error)]
pub enum VerificationError {
    /// The token signature could not be verified.
    #[error("Invalid signature")]
    InvalidSignature,

    /// The provided key algorithm is not supported or invalid.
    #[error("Invalid key algorithm")]
    InvalidKeyAlgorithm,

    /// The token format is invalid or could not be parsed.
    #[error("Invalid token")]
    InvalidToken,

    /// The token is missing the `kid` header.
    #[error("Missing 'kid' header in token")]
    NoKidHeader,

    /// No matching key was found for the specified `kid`.
    #[error("No matching public key found for 'kid'")]
    NoMatchingKid,

    /// Failed to decode or parse the public keys.
    #[error("Could not decode public keys")]
    CannotDecodePublicKeys,

    /// Failed to Base64 decode JWT.
    #[error("Could not decode public keys")]
    CannotDecodeJwt(#[from] base64::DecodeError),
}

#[derive(Debug, thiserror::Error)]
pub enum PublicKeysError {
    #[error("failed to fetch public keys from the identity provider: {0}")]
    FetchPublicKeys(reqwest::Error),

    #[error("missing 'Cache-Control' header in the response")]
    MissingCacheControlHeader,

    #[error("the 'max-age' directive is present but empty")]
    EmptyMaxAgeDirective,

    #[error("the 'max-age' directive is not a valid number")]
    InvalidMaxAgeValue,

    #[error("no 'max-age' directive found in 'Cache-Control' header")]
    MissingMaxAgeDirective,

    #[error("failed to parse one or more public keys: {0}")]
    PublicKeyParseError(reqwest::Error),
}
