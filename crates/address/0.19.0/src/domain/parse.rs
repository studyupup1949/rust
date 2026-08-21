use crate::ParseError::InvalidDomain;
use crate::{Domain, DomainRef, ParseError};
use std::str::FromStr;

impl FromStr for Domain {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Domain::try_from(s)
    }
}

impl<'a> TryFrom<&'a str> for DomainRef<'a> {
    type Error = ParseError;

    /// The `name` must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Domain::try_from` to parse mixed-case names.
    fn try_from(name: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(name.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for DomainRef<'a> {
    type Error = ParseError;

    /// The `name` must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Domain::try_from` to parse mixed-case names.
    fn try_from(name: &'a [u8]) -> Result<Self, Self::Error> {
        if Domain::is_valid_name(name, false) {
            let name: &str = unsafe { std::str::from_utf8_unchecked(name) };
            Ok(unsafe { DomainRef::new_unchecked(name) })
        } else {
            Err(InvalidDomain)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError::InvalidDomain;
    use crate::{Domain, DomainRef, ParseError};
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

    #[test]
    fn try_from_str() {
        let result: Result<DomainRef, ParseError> = DomainRef::try_from("localhost");
        assert_eq!(result, Ok(DomainRef::LOCALHOST));

        let result: Result<DomainRef, ParseError> = DomainRef::try_from("LocalHost");
        assert_eq!(result, Err(InvalidDomain));
    }

    #[test]
    fn try_from_slice() {
        let result: Result<DomainRef, ParseError> = DomainRef::try_from("localhost".as_bytes());
        assert_eq!(result, Ok(DomainRef::LOCALHOST));

        let result: Result<DomainRef, ParseError> = DomainRef::try_from("LocalHost".as_bytes());
        assert_eq!(result, Err(InvalidDomain));
    }
}
