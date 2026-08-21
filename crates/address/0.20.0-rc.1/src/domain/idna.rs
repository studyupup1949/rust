use crate::ParseError::InvalidDomain;
use crate::{Domain, DomainRef, ParseError};

impl Domain {
    //! International Domain Names

    /// Creates a domain from the Unicode `name`.
    ///
    /// Unicode labels are converted to their ASCII A-label form, so the domain will only contain ASCII.
    /// (example: `Bücher.example` becomes `xn--bcher-kva.example`)
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
        let test_cases: &[(&str, Result<Domain, ParseError>)] = &[
            ("Bücher.example", Ok(Domain::try_from("xn--bcher-kva.example").unwrap())),
            ("localhost", Ok(Domain::localhost())),
            ("", Err(InvalidDomain)),
        ];

        for (input, expected) in test_cases {
            let result: Result<Domain, ParseError> = Domain::from_unicode(input);
            assert_eq!(result, *expected, "input={}", input);
        }
    }

    #[test]
    fn to_unicode() {
        let test_cases: &[(Domain, Result<&str, ParseError>)] = &[
            (Domain::try_from("xn--bcher-kva.example").unwrap(), Ok("bücher.example")),
            (Domain::example(), Ok("example.com")),
            (Domain::try_from("xn--a.example").unwrap(), Err(InvalidDomain)),
        ];

        for (domain, expected) in test_cases {
            let result: Result<String, ParseError> = domain.to_unicode();
            assert_eq!(result, (*expected).map(String::from), "domain={}", domain);
        }
    }

    #[test]
    fn ref_to_unicode() {
        let domain: DomainRef = DomainRef::try_from("xn--bcher-kva.example").unwrap();
        assert_eq!(domain.to_unicode(), Ok("bücher.example".to_string()));
    }
}
