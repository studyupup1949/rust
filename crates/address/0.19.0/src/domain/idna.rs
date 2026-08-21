use crate::ParseError::InvalidDomain;
use crate::{Domain, DomainRef, ParseError};

impl Domain {
    //! International Domain Names

    /// Creates a domain from the Unicode `name`.
    ///
    /// Unicode labels are converted to their ASCII A-label form, so the domain will only contain
    /// ASCII. (example: `Bücher.example` becomes `xn--bcher-kva.example`)
    pub fn from_unicode(name: &str) -> Result<Self, ParseError> {
        let name: String = idna::domain_to_ascii(name).map_err(|_| InvalidDomain)?;
        Self::try_from(name).map_err(ParseError::from)
    }

    /// Converts the domain name to its Unicode representation.
    ///
    /// Returns an error if a label contains invalid punycode. (example: `xn--a.example`)
    pub fn to_unicode(&self) -> Result<String, ParseError> {
        self.to_ref().to_unicode()
    }
}

impl<'a> DomainRef<'a> {
    //! International Domain Names

    /// Converts the domain name to its Unicode representation.
    ///
    /// Returns an error if a label contains invalid punycode. (example: `xn--a.example`)
    pub fn to_unicode(self) -> Result<String, ParseError> {
        let (name, result) = idna::domain_to_unicode(self.name());
        result.map_err(|_| InvalidDomain)?;
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use crate::ParseError::InvalidDomain;
    use crate::{Domain, DomainRef, ParseError};

    #[test]
    fn from_unicode() {
        let result: Result<Domain, ParseError> = Domain::from_unicode("Bücher.example");
        let expected: Domain = Domain::try_from("xn--bcher-kva.example").unwrap();
        assert_eq!(result, Ok(expected));

        let result: Result<Domain, ParseError> = Domain::from_unicode("localhost");
        assert_eq!(result, Ok(Domain::localhost()));

        let result: Result<Domain, ParseError> = Domain::from_unicode("");
        assert_eq!(result, Err(InvalidDomain));
    }

    #[test]
    fn to_unicode() {
        let domain: Domain = Domain::try_from("xn--bcher-kva.example").unwrap();
        assert_eq!(domain.to_unicode(), Ok("bücher.example".to_string()));

        let domain: Domain = Domain::example();
        assert_eq!(domain.to_unicode(), Ok("example.com".to_string()));

        let domain: Domain = Domain::try_from("xn--a.example").unwrap();
        assert_eq!(domain.to_unicode(), Err(InvalidDomain));
    }

    #[test]
    fn ref_to_unicode() {
        let domain: DomainRef = DomainRef::try_from("xn--bcher-kva.example").unwrap();
        assert_eq!(domain.to_unicode(), Ok("bücher.example".to_string()));
    }
}
