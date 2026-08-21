use crate::ParseError::InvalidDomain;
use crate::{DomainRef, ParseError};

/// A domain name.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
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
    /// The `name` must be valid.
    pub unsafe fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        let name: String = name.into();

        debug_assert!(Self::is_valid_name_str(name.as_str(), false));

        Self { name }
    }
}

impl TryFrom<String> for Domain {
    type Error = (ParseError, String);

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if Self::is_valid_name_str(name.as_str(), false) {
            Ok(Self { name })
        } else if Self::is_valid_name_str(name.as_str(), true) {
            let mut name: String = name;
            name.make_ascii_lowercase();
            Ok(Self { name })
        } else {
            Err((InvalidDomain, name))
        }
    }
}

impl TryFrom<&str> for Domain {
    type Error = ParseError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        if Self::is_valid_name_str(name, false) {
            let name: String = name.to_string();
            Ok(Self { name })
        } else if Self::is_valid_name_str(name, true) {
            let name: String = name.to_ascii_lowercase();
            Ok(Self { name })
        } else {
            Err(InvalidDomain)
        }
    }
}

impl TryFrom<Vec<u8>> for Domain {
    type Error = (ParseError, Vec<u8>);

    fn try_from(name: Vec<u8>) -> Result<Self, Self::Error> {
        if Self::is_valid_name(name.as_slice(), false) {
            let name: String = unsafe { String::from_utf8_unchecked(name) };
            Ok(Self { name })
        } else if Self::is_valid_name(name.as_slice(), true) {
            let mut name: String = unsafe { String::from_utf8_unchecked(name) };
            name.make_ascii_lowercase();
            Ok(Self { name })
        } else {
            Err((InvalidDomain, name))
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

impl<'a> From<DomainRef<'a>> for Domain {
    fn from(domain: DomainRef) -> Self {
        domain.to_domain()
    }
}

impl From<Domain> for String {
    fn from(domain: Domain) -> Self {
        domain.name
    }
}

impl Domain {
    //! Properties

    /// Gets the name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError::InvalidDomain;
    use crate::{Domain, DomainRef, ParseError};

    #[test]
    fn specials() {
        assert_eq!(Domain::localhost().name, "localhost");
        assert_eq!(Domain::example().name, "example.com");
    }

    #[test]
    fn try_from_string() {
        let result: Result<Domain, (ParseError, String)> =
            Domain::try_from("localhost".to_string());
        assert_eq!(result.map_err(|e| e.0), Ok(Domain::localhost()));

        let result: Result<Domain, (ParseError, String)> =
            Domain::try_from("LocalHost".to_string());
        assert_eq!(result.map_err(|e| e.0), Ok(Domain::localhost()));

        let result: Result<Domain, (ParseError, String)> =
            Domain::try_from("Local!Host".to_string());
        assert_eq!(result.map_err(|e| e.0), Err(InvalidDomain));
    }

    #[test]
    fn try_from_str() {
        let result: Result<Domain, ParseError> = Domain::try_from("localhost");
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ParseError> = Domain::try_from("LocalHost");
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ParseError> = Domain::try_from("Local!Host");
        assert_eq!(result, Err(InvalidDomain));
    }

    #[test]
    fn try_from_vec() {
        let result: Result<Domain, (ParseError, Vec<u8>)> =
            Domain::try_from(Vec::from("localhost"));
        assert_eq!(result.map_err(|e| e.0), Ok(Domain::localhost()));

        let result: Result<Domain, (ParseError, Vec<u8>)> =
            Domain::try_from(Vec::from("LocalHost"));
        assert_eq!(result.map_err(|e| e.0), Ok(Domain::localhost()));

        let result: Result<Domain, (ParseError, Vec<u8>)> =
            Domain::try_from(Vec::from("Local!Host"));
        assert_eq!(result.map_err(|e| e.0), Err(InvalidDomain));
    }

    #[test]
    fn try_from_slice() {
        let result: Result<Domain, ParseError> = Domain::try_from("localhost".as_bytes());
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ParseError> = Domain::try_from("LocalHost".as_bytes());
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ParseError> = Domain::try_from("Local!Host".as_bytes());
        assert_eq!(result, Err(InvalidDomain));
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
    fn properties() {
        let domain: Domain = Domain::localhost();
        assert_eq!(domain.name(), "localhost");
    }
}
