use std::fmt;
use std::io;

#[derive(Debug)]
pub enum AamlError {
    IoError(io::Error),
    ParseError {
        line: usize,
        content: String,
        details: String,
    },
    NotFound(String),
    InvalidValue(String),
    InvalidType {
        type_name: String,
        details: String,
    },
    DirectiveError(String, String),
}

impl fmt::Display for AamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AamlError::IoError(err) => write!(f, "IO Error: {}", err),
            AamlError::ParseError { line, content, details } => {
                write!(f, "Parse Error at line {}: '{}'. Reason: {}", line, content, details)
            }
            AamlError::NotFound(key) => write!(f, "Key not found: '{}'", key),
            AamlError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            AamlError::InvalidType { type_name, details } => {
                write!(f, "Invalid type '{}': {}", type_name, details)
            }
            AamlError::DirectiveError(cmd, msg) => {
                write!(f, "Directive '@{}' error: {}", cmd, msg)
            }
        }
    }
}

impl std::error::Error for AamlError {}

impl From<io::Error> for AamlError {
    fn from(err: io::Error) -> Self {
        AamlError::IoError(err)
    }
}