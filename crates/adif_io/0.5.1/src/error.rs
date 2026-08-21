use std::{error, fmt};

/// All errors regarding ADIF-IO
#[derive(Debug, PartialEq)]
pub enum Error {
    DeserializeMissingHeader,
    DeserializeValueLength(String),
    DeserializeOutOfBounds(usize),
    DeserializeRecord(String),
    InvalidGridsquare(String),
    InvalidCoordinate(f64, f64),

}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}


#[cfg(test)]
mod test {
    use crate::Error;

    #[test]
    fn test_400_errors() {
        assert_eq!(Error::DeserializeMissingHeader.to_string(), "DeserializeMissingHeader")
    }
}