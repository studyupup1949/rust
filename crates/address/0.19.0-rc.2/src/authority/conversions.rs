use crate::{Authority, AuthorityRef, Endpoint, EndpointRef, Host, HostRef, SocketAddress};

impl Authority {
    //! Conversions

    /// Converts the authority to an authority reference.
    #[must_use]
    pub fn to_ref(&self) -> AuthorityRef<'_> {
        AuthorityRef::new(self.host(), self.port())
    }

    /// Converts the authority to an optional endpoint.
    #[must_use]
    pub fn to_endpoint(self) -> Option<Endpoint> {
        let (host, port) = self.into();
        if let Host::Name(domain) = host {
            Some(Endpoint::new(domain, port))
        } else {
            None
        }
    }

    /// Converts the authority to an optional socket address.
    #[must_use]
    pub fn to_socket(self) -> Option<SocketAddress> {
        let (host, port) = self.into();
        if let Host::Address(ip) = host {
            Some(SocketAddress::new(ip, port))
        } else {
            None
        }
    }
}

impl<'a> AuthorityRef<'a> {
    //! Conversions

    /// Converts the authority reference to an authority.
    #[must_use]
    pub fn to_authority(self) -> Authority {
        Authority::new(self.host().to_host(), self.port())
    }

    /// Converts the authority reference to an optional endpoint reference.
    #[must_use]
    pub const fn to_endpoint_ref(self) -> Option<EndpointRef<'a>> {
        if let HostRef::Name(domain) = self.host() {
            Some(EndpointRef::new(domain, self.port()))
        } else {
            None
        }
    }

    /// Converts the authority reference to an optional socket address.
    #[must_use]
    pub const fn to_socket(self) -> Option<SocketAddress> {
        if let HostRef::Address(ip) = self.host() {
            Some(SocketAddress::new(ip, self.port()))
        } else {
            None
        }
    }
}

impl<'a> From<&'a Authority> for AuthorityRef<'a> {
    fn from(authority: &'a Authority) -> Self {
        authority.to_ref()
    }
}

impl<'a> PartialEq<AuthorityRef<'a>> for Authority {
    fn eq(&self, other: &AuthorityRef<'a>) -> bool {
        self.to_ref() == *other
    }
}

impl<'a> PartialEq<Authority> for AuthorityRef<'a> {
    fn eq(&self, other: &Authority) -> bool {
        *self == other.to_ref()
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
