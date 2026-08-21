use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("Solution failed: {msg}")]
    ExecutionFailed {
        msg: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Solution timed out after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("Invalid solution result: {0}")]
    InvalidResult(String),

    #[error("Solution not implemented")]
    NotImplemented,
}
