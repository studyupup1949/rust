use crate::ParseError::InvalidDomain;
use crate::{Domain, DomainRef, ParseError};

impl Domain {
    //! International Domain Names

    /// Creates a domain from the unicode `name`.
    ///
    /// Unicode labels are converted to their ASCII A-label form, so the domain will only contain
    /// ASCII. (example: `Bücher.example` becomes `xn--bcher-kva.example`)
    pub fn from_unicode(name: &str) -> Result<Self, ParseError> {
        let name: String = idna::domain_to_ascii(name).map_err(|_| InvalidDomain)?;
        Self::try_from(name).map_err(ParseError::from)
    }

    /// Converts the domain name to its unicode representation.
    pub fn to_unicode(&self) -> String {
        self.to_ref().to_unicode()
    }
}

impl<'a> DomainRef<'a> {
    //! International Domain Names

    /// Converts the domain name to its unicode representation.
    pub fn to_unicode(self) -> String {
        idna::domain_to_unicode(self.name()).0
    }
}
