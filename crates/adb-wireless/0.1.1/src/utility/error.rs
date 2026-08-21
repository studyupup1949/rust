use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("ADB is not found")]
    AdbNotFound,
    #[error("ADB server error. {0}")]
    AdbServerError(#[from] std::io::Error),
    #[error("Invalid port mapping: {0}")]
    InvalidPortMapping(String),
    #[error("Failed to generate QR code. {0}")]
    QrCodeError(#[from] qr2term::QrError),
    #[error("MDNS error. {0}")]
    MdnsError(#[from] mdns_sd::Error),
    #[error("An unexpected error occurred: {0}")]
    UnexpectedError(String),
}
