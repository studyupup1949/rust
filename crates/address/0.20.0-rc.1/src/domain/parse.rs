use crate::ParseError::InvalidDomain;
use crate::{Domain, DomainRef, ParseError};
use std::str::FromStr;

impl FromStr for Domain {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl<'a> TryFrom<&'a str> for DomainRef<'a> {
    type Error = ParseError;

    /// The `name` must already be lowercase, since a borrowed name cannot be normalized. Use `Domain::try_from` to
    /// parse mixed-case names.
    fn try_from(name: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(name.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for DomainRef<'a> {
    type Error = ParseError;

    /// The `name` must already be lowercase, since a borrowed name cannot be normalized. Use `Domain::try_from` to
    /// parse mixed-case names.
    fn try_from(name: &'a [u8]) -> Result<Self, Self::Error> {
        if Domain::is_valid_name(name, false) {
            let name: &str = std::str::from_utf8(name).map_err(|_| InvalidDomain)?;
            Ok(unsafe { Self::new_unchecked(name) })
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
        let test_cases: &[(&str, Result<Domain, ParseError>)] = &[
            ("localhost", Ok(Domain::localhost())),
            ("LocalHost", Ok(Domain::localhost())),
            ("Local!Host", Err(InvalidDomain)),
        ];

        for (input, expected) in test_cases {
            let result: Result<Domain, ParseError> = Domain::from_str(input);
            assert_eq!(result, *expected, "input={}", input);
        }
    }

    #[test]
    fn try_from_str() {
        let test_cases: &[(&str, Result<DomainRef, ParseError>)] = &[
            ("localhost", Ok(DomainRef::LOCALHOST)),
            ("LocalHost", Err(InvalidDomain)),
        ];

        for (input, expected) in test_cases {
            let result: Result<DomainRef, ParseError> = DomainRef::try_from(*input);
            assert_eq!(result, *expected, "input={}", input);
        }
    }

    #[test]
    fn try_from_slice() {
        let test_cases: &[(&str, Result<DomainRef, ParseError>)] = &[
            ("localhost", Ok(DomainRef::LOCALHOST)),
            ("LocalHost", Err(InvalidDomain)),
        ];

        for (input, expected) in test_cases {
            let result: Result<DomainRef, ParseError> = DomainRef::try_from(input.as_bytes());
            assert_eq!(result, *expected, "input={}", input);
        }
    }
}
