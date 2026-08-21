use crate::{Domain, ParseError};
use std::str::FromStr;

impl FromStr for Domain {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Domain::try_from(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError::InvalidDomain;
    use crate::{Domain, ParseError};
    use std::str::FromStr;

    #[test]
    fn from_str() {
        let result: Result<Domain, ParseError> = Domain::from_str("localhost");
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ParseError> = Domain::from_str("LocalHost");
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ParseError> = Domain::from_str("Local!Host");
        assert_eq!(result, Err(InvalidDomain));
    }
}
