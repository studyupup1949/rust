use crate::ParseError::InvalidDomain;
use crate::{Domain, ParseError};

/// A domain name reference.
///
/// See `Domain` for the domain name validation rules. A borrowed name cannot be normalized, so
/// the name must already be lowercase.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct DomainRef<'a> {
    name: &'a str,
}

impl<'a> DomainRef<'a> {
    //! Special Domains

    /// The `localhost` domain reference.
    pub const LOCALHOST: Self = Self { name: "localhost" };

    /// The `example.com` domain reference.
    pub const EXAMPLE: Self = Self {
        name: "example.com",
    };
}

impl<'a> DomainRef<'a> {
    //! Construction

    /// Creates a new domain reference.
    ///
    /// # Safety
    /// The `name` must be valid and lowercase. Validity is a struct invariant that unsafe code
    /// may rely on.
    pub unsafe fn new_unchecked(name: &'a str) -> Self {
        debug_assert!(Domain::is_valid_name_str(name, false));

        Self { name }
    }
}

impl<'a> TryFrom<&'a str> for DomainRef<'a> {
    type Error = ParseError;

    /// The `name` must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Domain::try_from` to parse mixed-case names.
    fn try_from(name: &'a str) -> Result<Self, Self::Error> {
        if Domain::is_valid_name_str(name, false) {
            Ok(Self { name })
        } else {
            Err(InvalidDomain)
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for DomainRef<'a> {
    type Error = ParseError;

    /// The `name` must already be lowercase, since a borrowed name cannot be normalized. Use
    /// `Domain::try_from` to parse mixed-case names.
    fn try_from(name: &'a [u8]) -> Result<Self, Self::Error> {
        if Domain::is_valid_name(name, false) {
            let name: &str = unsafe { std::str::from_utf8_unchecked(name) };
            Ok(Self { name })
        } else {
            Err(InvalidDomain)
        }
    }
}

impl<'a> DomainRef<'a> {
    //! Properties

    /// Gets the name.
    pub const fn name(self) -> &'a str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError::InvalidDomain;
    use crate::{DomainRef, ParseError};

    #[test]
    fn specials() {
        assert_eq!(DomainRef::LOCALHOST.name, "localhost");
        assert_eq!(DomainRef::EXAMPLE.name, "example.com");
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

    #[test]
    fn properties() {
        let domain: DomainRef = DomainRef::LOCALHOST;
        assert_eq!(domain.name(), "localhost");
    }
}
