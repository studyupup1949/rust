use crate::ParseError::InvalidDomain;
use crate::{DomainRef, InvalidDomainName, ParseError};

/// A domain name.
///
/// Domain names are lowercase ASCII letters, digits, and dashes: dot-separated labels that must not start or end
/// with a dash. (see `Domain::is_valid_name`) Mixed-case input is normalized to lowercase when parsed.
#[must_use]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Domain {
    name: String,
}

impl Domain {
    //! Special Domains

    /// Creates the `localhost` domain.
    pub fn localhost() -> Self {
        DomainRef::LOCALHOST.to_domain()
    }

    /// Creates the `example.com` domain.
    pub fn example() -> Self {
        DomainRef::EXAMPLE.to_domain()
    }
}

impl Domain {
    //! Construction

    /// Creates a new domain.
    ///
    /// # Safety
    /// The `name` must be valid and lowercase. Validity is a struct invariant that unsafe code may rely on.
    pub unsafe fn new_unchecked<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        let name: String = name.into();

        debug_assert!(Self::is_valid_name_str(name.as_str(), false));

        Self { name }
    }
}

impl TryFrom<String> for Domain {
    type Error = InvalidDomainName<String>;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if Self::is_valid_name_str(name.as_str(), false) {
            Ok(Self { name })
        } else if Self::is_valid_name_str(name.as_str(), true) {
            let mut name: String = name;
            name.make_ascii_lowercase();
            Ok(Self { name })
        } else {
            Err(InvalidDomainName::new(name))
        }
    }
}

impl TryFrom<&str> for Domain {
    type Error = ParseError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        Self::try_from(name.as_bytes())
    }
}

impl TryFrom<Vec<u8>> for Domain {
    type Error = InvalidDomainName<Vec<u8>>;

    fn try_from(name: Vec<u8>) -> Result<Self, Self::Error> {
        if Self::is_valid_name(name.as_slice(), false) {
            let name: String = unsafe { String::from_utf8_unchecked(name) };
            Ok(Self { name })
        } else if Self::is_valid_name(name.as_slice(), true) {
            let mut name: String = unsafe { String::from_utf8_unchecked(name) };
            name.make_ascii_lowercase();
            Ok(Self { name })
        } else {
            Err(InvalidDomainName::new(name))
        }
    }
}

impl TryFrom<&[u8]> for Domain {
    type Error = ParseError;

    fn try_from(name: &[u8]) -> Result<Self, Self::Error> {
        if Self::is_valid_name(name, false) {
            let name: &str = unsafe { std::str::from_utf8_unchecked(name) };
            let name: String = name.to_string();
            Ok(Self { name })
        } else if Self::is_valid_name(name, true) {
            let name: &str = unsafe { std::str::from_utf8_unchecked(name) };
            let name: String = name.to_ascii_lowercase();
            Ok(Self { name })
        } else {
            Err(InvalidDomain)
        }
    }
}

impl From<Domain> for String {
    fn from(domain: Domain) -> Self {
        domain.name
    }
}

impl<'a> From<DomainRef<'a>> for Domain {
    fn from(domain: DomainRef<'a>) -> Self {
        domain.to_domain()
    }
}

impl<'a> PartialEq<DomainRef<'a>> for Domain {
    fn eq(&self, other: &DomainRef<'a>) -> bool {
        self.to_ref() == *other
    }
}

impl Domain {
    //! Properties

    /// Gets the name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError::InvalidDomain;
    use crate::{Domain, DomainRef, InvalidDomainName, ParseError};

    #[test]
    fn specials() {
        assert_eq!(Domain::localhost().name, "localhost");
        assert_eq!(Domain::example().name, "example.com");
    }

    #[test]
    fn try_from_string() {
        let test_cases: &[(&str, Result<Domain, &str>)] = &[
            ("localhost", Ok(Domain::localhost())),
            ("LocalHost", Ok(Domain::localhost())),
            ("Local!Host", Err("Local!Host")),
        ];

        for (input, expected) in test_cases {
            let result: Result<Domain, InvalidDomainName<String>> = Domain::try_from(input.to_string());
            let result: Result<Domain, String> = result.map_err(|e| e.into_name());
            assert_eq!(result, expected.clone().map_err(String::from), "input={}", input);
        }
    }

    #[test]
    fn try_from_str() {
        let test_cases: &[(&str, Result<Domain, ParseError>)] = &[
            ("localhost", Ok(Domain::localhost())),
            ("LocalHost", Ok(Domain::localhost())),
            ("Local!Host", Err(InvalidDomain)),
        ];

        for (input, expected) in test_cases {
            let result: Result<Domain, ParseError> = Domain::try_from(*input);
            assert_eq!(result, *expected, "input={}", input);
        }
    }

    #[test]
    fn try_from_vec() {
        let test_cases: &[(&str, Result<Domain, &str>)] = &[
            ("localhost", Ok(Domain::localhost())),
            ("LocalHost", Ok(Domain::localhost())),
            ("Local!Host", Err("Local!Host")),
        ];

        for (input, expected) in test_cases {
            let result: Result<Domain, InvalidDomainName<Vec<u8>>> = Domain::try_from(Vec::from(*input));
            let result: Result<Domain, Vec<u8>> = result.map_err(|e| e.into_name());
            assert_eq!(result, expected.clone().map_err(Vec::from), "input={}", input);
        }
    }

    #[test]
    fn try_from_slice() {
        let test_cases: &[(&str, Result<Domain, ParseError>)] = &[
            ("localhost", Ok(Domain::localhost())),
            ("LocalHost", Ok(Domain::localhost())),
            ("Local!Host", Err(InvalidDomain)),
        ];

        for (input, expected) in test_cases {
            let result: Result<Domain, ParseError> = Domain::try_from(input.as_bytes());
            assert_eq!(result, *expected, "input={}", input);
        }
    }

    #[test]
    fn from_ref() {
        let domain: DomainRef = DomainRef::LOCALHOST;

        let result: Domain = domain.into();
        let expected: Domain = Domain::localhost();
        assert_eq!(result, expected);
    }

    #[test]
    fn deconstruction() {
        let domain: Domain = Domain::localhost();

        let result: String = domain.into();
        let expected: &str = "localhost";
        assert_eq!(result, expected);
    }

    #[test]
    fn equality() {
        let domain: Domain = Domain::localhost();
        assert_eq!(domain, DomainRef::LOCALHOST);
        assert_ne!(domain, DomainRef::EXAMPLE);
    }

    #[test]
    fn properties() {
        let domain: Domain = Domain::localhost();
        assert_eq!(domain.name(), "localhost");
    }
}
