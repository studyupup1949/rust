use thiserror::Error;

pub type Result<T> = std::result::Result<T, AdversariaError>;

#[derive(Error, Debug)]
pub enum AdversariaError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Suite error: {0}")]
    Suite(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid attack payload: {0}")]
    InvalidPayload(String),

    #[error("Report not found: {0}")]
    ReportNotFound(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
