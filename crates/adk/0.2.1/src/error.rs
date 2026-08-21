use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Tool execution error: {0}")]
    ToolError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Context error: {0}")]
    ContextError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("OpenAI API error: {0}")]
    OpenAIError(#[from] async_openai::error::OpenAIError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type AgentResult<T> = Result<T, AgentError>;
