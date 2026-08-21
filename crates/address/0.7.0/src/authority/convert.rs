use crate::{Authority, AuthorityRef};

impl Authority {
    //! Conversions

    /// Converts the authority to an authority reference.
    pub fn to_ref(&self) -> AuthorityRef {
        AuthorityRef::new(self.host(), self.port())
    }
}

impl<'a> AuthorityRef<'a> {
    //! Conversions

    /// Converts the authority reference to an authority.
    pub fn to_authority(&self) -> Authority {
        Authority::new(self.host().to_host(), self.port())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, AuthorityRef, Domain, DomainRef, HostRef};

    #[test]
    fn authority() {
        let authority: Authority = Domain::localhost().to_endpoint(80).to_authority();

        let result: AuthorityRef = authority.to_ref();
        let expected: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn authority_ref() {
        let result: Authority = DomainRef::LOCALHOST
            .to_endpoint(80)
            .to_authority()
            .to_authority();
        let expected: Authority = Authority::new(Domain::localhost().to_host(), 80);
        assert_eq!(result, expected);
    }
}
