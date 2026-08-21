use crate::{Authority, AuthorityRef};

impl Authority {
    //! Conversions

    /// Converts the authority to an authority reference.
    pub fn to_ref(&self) -> AuthorityRef<'_> {
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
    use crate::{Authority, AuthorityRef, Domain, DomainRef, Host, HostRef};

    #[test]
    fn authority_to_ref() {
        let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);

        let result: AuthorityRef = authority.to_ref();
        let expected: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_authority() {
        let host: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        let authority: AuthorityRef = AuthorityRef::new(host, 80);

        let result: Authority = authority.to_authority();
        let expected: Authority = Authority::new(host.to_host(), 80);
        assert_eq!(result, expected);
    }
}
