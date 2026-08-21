/// A crate-wide result type alias using the custom [`Error`] enum.
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for Firebase authentication-related failures.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Errors that occur while fetching or parsing Firebase public keys.
    #[error(transparent)]
    PublicKeysError(#[from] crate::jwk::PublicKeysError),

    /// Errors that occur during JWT verification or claim validation.
    #[error(transparent)]
    VerificationError(#[from] crate::jwk::VerificationError),

    /// Errors that occur during Firebase identity provider claim validation.
    #[cfg(feature = "idp")]
    #[error(transparent)]
    IdpError(#[from] crate::firebase::FirebaseIdpError),
}
