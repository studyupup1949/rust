#[derive(thiserror::Error, Debug)]
pub enum Hj212Error {
    #[error("frame missing prefix ##")]
    MissingPrefix,

    #[error("frame missing CP section")]
    MissingCp,

    #[error("invalid length field")]
    InvalidLength,

    #[error("invalid CRC field")]
    InvalidCrc,

    #[error("frame missing suffix CRLF")]
    MissingSuffix,

    #[error("CRC mismatch: expected {expected}, got {got}")]
    CrcMismatch { expected: String, got: String },

    #[error("invalid DataTime format")]
    InvalidDataTime,

    #[error("invalid encrypted data format")]
    InvalidEncryptedDataFormat,

    #[error("invalid encrypted data length")]
    InvalidEncryptedDataLength,

    #[error("non-ascii data cannot be encrypted")]
    NonAsciiForEncryption,
}
