use thiserror::Error;

#[derive(Debug, Error)]
pub enum AddMcpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Invalid source: {0}")]
    InvalidSource(String),

    #[error("Unknown agent: {0}")]
    UnknownAgent(String),

    #[error("Config path not found for {agent} ({scope})")]
    ConfigPathNotFound { agent: String, scope: String },

    #[error("Cannot infer server name from source: {0}")]
    CannotInferName(String),

    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AddMcpError>;
