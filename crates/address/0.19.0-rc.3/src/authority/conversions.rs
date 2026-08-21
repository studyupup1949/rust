use crate::{Authority, AuthorityRef, Endpoint, EndpointRef, Host, HostRef, SocketAddress};

impl Authority {
    //! Conversions

    /// Converts the authority to an authority reference.
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

#[cfg(test)]
mod tests {
    use crate::{
        Authority, AuthorityRef, Domain, DomainRef, Endpoint, EndpointRef, Host, HostRef,
        IPv4Address, SocketAddress,
    };

    #[test]
    fn authority_to_ref() {
        let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);

        let result: AuthorityRef = authority.to_ref();
        let expected: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn authority_to_endpoint() {
        let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
        let result: Option<Endpoint> = authority.to_endpoint();
        let expected: Option<Endpoint> = Some(Endpoint::new(Domain::localhost(), 80));
        assert_eq!(result, expected);

        let authority: Authority = Authority::new(IPv4Address::LOCALHOST.to_host(), 80);
        let result: Option<Endpoint> = authority.to_endpoint();
        let expected: Option<Endpoint> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn authority_to_socket() {
        let authority: Authority = Authority::new(IPv4Address::LOCALHOST.to_host(), 80);
        let result: Option<SocketAddress> = authority.to_socket();
        let expected: Option<SocketAddress> = Some(IPv4Address::LOCALHOST.to_ip().to_socket(80));
        assert_eq!(result, expected);

        let authority: Authority = Authority::new(Host::Name(Domain::localhost()), 80);
        let result: Option<SocketAddress> = authority.to_socket();
        let expected: Option<SocketAddress> = None;
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

    #[test]
    fn ref_to_endpoint() {
        let authority: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        let result: Option<EndpointRef> = authority.to_endpoint_ref();
        let expected: Option<EndpointRef> = Some(EndpointRef::new(DomainRef::LOCALHOST, 80));
        assert_eq!(result, expected);

        let authority: AuthorityRef = AuthorityRef::new(IPv4Address::LOCALHOST.to_host_ref(), 80);
        let result: Option<EndpointRef> = authority.to_endpoint_ref();
        let expected: Option<EndpointRef> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_socket() {
        let authority: AuthorityRef = AuthorityRef::new(IPv4Address::LOCALHOST.to_host_ref(), 80);
        let result: Option<SocketAddress> = authority.to_socket();
        let expected: Option<SocketAddress> = Some(IPv4Address::LOCALHOST.to_ip().to_socket(80));
        assert_eq!(result, expected);

        let authority: AuthorityRef = AuthorityRef::new(HostRef::Name(DomainRef::LOCALHOST), 80);
        let result: Option<SocketAddress> = authority.to_socket();
        let expected: Option<SocketAddress> = None;
        assert_eq!(result, expected);
    }
}
