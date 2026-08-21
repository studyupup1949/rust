pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PublicKeysError(#[from] crate::jwk::PublicKeysError),

    #[error(transparent)]
    VerificationError(#[from] crate::jwk::VerificationError),
}
