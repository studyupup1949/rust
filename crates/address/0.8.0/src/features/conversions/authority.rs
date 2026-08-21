use crate::{Authority, AuthorityRef, Endpoint, EndpointRef, Host, HostRef, SocketAddress};

impl Authority {
    /// Converts the authority to an authority reference.
    pub fn to_ref(&self) -> AuthorityRef {
        AuthorityRef::new(self.host(), self.port())
    }

    /// Converts the authority to an optional endpoint.
    pub fn to_endpoint(self) -> Option<Endpoint> {
        let (host, port) = self.into();
        match host {
            Host::Name(domain) => Some(Endpoint::new(domain, port)),
            _ => None,
        }
    }

    /// Converts the authority to an optional socket address.
    pub fn to_socket(self) -> Option<SocketAddress> {
        let (host, port) = self.into();
        match host {
            Host::Address(ip) => Some(SocketAddress::new(ip, port)),
            _ => None,
        }
    }
}

impl<'a> AuthorityRef<'a> {
    /// Converts the authority reference to an authority.
    pub fn to_authority(&self) -> Authority {
        Authority::new(self.host().to_host(), self.port())
    }

    /// Converts the authority reference to an optional endpoint reference.
    pub fn to_endpoint(&self) -> Option<EndpointRef> {
        match self.host() {
            HostRef::Name(domain) => Some(EndpointRef::new(domain, self.port())),
            _ => None,
        }
    }

    /// Converts the authority to an optional socket address.
    pub fn to_socket(self) -> Option<SocketAddress> {
        match self.host() {
            HostRef::Address(ip) => Some(SocketAddress::new(ip, self.port())),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Authority, AuthorityRef, Domain, DomainRef, Endpoint, EndpointRef, HostRef, IPv4Address,
        SocketAddress,
    };

    #[test]
    fn authority_to_ref() {
        let authority: Authority = IPv4Address::LOCALHOST.to_host().to_authority(80);
        let result: AuthorityRef = authority.to_ref();
        let expected: AuthorityRef = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(result, expected);
    }

    #[test]
    fn authority_to_endpoint() {
        let authority: Authority = IPv4Address::LOCALHOST.to_host().to_authority(80);
        let result: Option<Endpoint> = authority.to_endpoint();
        let expected: Option<Endpoint> = None;
        assert_eq!(result, expected);

        let authority: Authority = Domain::localhost().to_host().to_authority(80);
        let result: Option<Endpoint> = authority.to_endpoint();
        let expected: Option<Endpoint> = Some(Domain::localhost().to_endpoint(80));
        assert_eq!(result, expected);
    }

    #[test]
    fn authority_to_socket() {
        let authority: Authority = IPv4Address::LOCALHOST.to_host().to_authority(80);
        let result: Option<SocketAddress> = authority.to_socket();
        let expected: Option<SocketAddress> =
            Some(IPv4Address::LOCALHOST.to_socket(80).to_socket());
        assert_eq!(result, expected);

        let authority: Authority = Domain::localhost().to_host().to_authority(80);
        let result: Option<SocketAddress> = authority.to_socket();
        let expected: Option<SocketAddress> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_authority() {
        let authority: AuthorityRef = (IPv4Address::LOCALHOST, 80).into();
        let result: Authority = authority.to_authority();
        let expected: Authority = (IPv4Address::LOCALHOST, 80).into();
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_endpoint() {
        let authority: AuthorityRef = (IPv4Address::LOCALHOST, 80).into();
        let result: Option<EndpointRef> = authority.to_endpoint();
        let expected: Option<EndpointRef> = None;
        assert_eq!(result, expected);

        let domain: DomainRef = DomainRef::LOCALHOST;
        let host: HostRef = domain.to_host();
        let authority: AuthorityRef = host.to_authority(80);
        let result: Option<EndpointRef> = authority.to_endpoint();
        let expected: Option<EndpointRef> = Some(DomainRef::LOCALHOST.to_endpoint(80));
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_socket() {
        let authority: AuthorityRef = (IPv4Address::LOCALHOST, 80).into();
        let result: Option<SocketAddress> = authority.to_socket();
        let expected: Option<SocketAddress> =
            Some(IPv4Address::LOCALHOST.to_socket(80).to_socket());
        assert_eq!(result, expected);

        let domain: DomainRef = DomainRef::LOCALHOST;
        let host: HostRef = domain.to_host();
        let authority: AuthorityRef = host.to_authority(80);
        let result: Option<SocketAddress> = authority.to_socket();
        let expected: Option<SocketAddress> = None;
        assert_eq!(result, expected);
    }
}
