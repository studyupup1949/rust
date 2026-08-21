use tonic::{Code, Status};

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("{message}")]
pub struct AdaError {
    pub code: Code,
    pub message: String,
}

impl From<Status> for AdaError {
    fn from(status: Status) -> Self {
        Self {
            code: status.code(),
            message: status.message().to_owned(),
        }
    }
}

impl From<tonic::transport::Error> for AdaError {
    fn from(error: tonic::transport::Error) -> Self {
        Self {
            code: Code::Unavailable,
            message: error.to_string(),
        }
    }
}

impl From<tonic::metadata::errors::InvalidMetadataValue> for AdaError {
    fn from(error: tonic::metadata::errors::InvalidMetadataValue) -> Self {
        Self {
            code: Code::InvalidArgument,
            message: error.to_string(),
        }
    }
}

pub(crate) fn retryable(error: &AdaError) -> bool {
    matches!(
        error.code,
        Code::Aborted | Code::Internal | Code::ResourceExhausted | Code::Unavailable
    )
}
