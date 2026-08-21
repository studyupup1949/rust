use chumsky::prelude::SimpleSpan;
use std::error;
use std::fmt;

/// Top level error type
#[derive(Debug, PartialEq, Eq, Default)]
pub enum AcfError {
    /// An error occurred reading a file
    Read(String),

    /// An error occurring during parsing (with specific sub-type)
    Parse(ParseError),

    /// An unknown/uncategorized error
    #[default]
    Unknown,
}

impl fmt::Display for AcfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AcfError::Read(val) => write!(f, "failed to read '{}'", &val),
            AcfError::Parse(..) => write!(f, "the provided input could not be parsed"),
            AcfError::Unknown => write!(f, "an unknown error occurred"),
        }
    }
}

impl error::Error for AcfError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            AcfError::Read(..) => None,
            AcfError::Parse(ref e) => Some(e),
            AcfError::Unknown => None,
        }
    }
}

/// Representation of a filesystem error
#[derive(Debug, PartialEq, Eq, Default)]
pub enum IOError {
    /// An unknown/uncategorized error
    #[default]
    Unknown,
}

impl fmt::Display for IOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IOError::Unknown => write!(f, "an unknown I/O error occurred"),
        }
    }
}

impl error::Error for IOError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            IOError::Unknown => None,
        }
    }
}

/// Representation of a parser error
#[derive(Debug, PartialEq, Eq, Default)]
pub enum ParseError {
    /// A closing brace was not found
    ExpectedClosingBrace(SimpleSpan),

    /// An unknown/uncategorized error
    #[default]
    Unknown,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::ExpectedClosingBrace(val) => {
                write!(f, "expected a closing brace within '{}'", &val)
            }
            ParseError::Unknown => write!(f, "an unknown parsing error occurred"),
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ParseError::ExpectedClosingBrace(_) => None,
            ParseError::Unknown => None,
        }
    }
}
