use crate::{Authority, AuthorityRef, Host, HostRef};

impl Host {
    //! Conversions

    /// Converts the host to a host reference.
    pub fn to_ref(&self) -> HostRef {
        match self {
            Self::Name(domain) => HostRef::Name(domain.to_ref()),
            Self::Address(ip) => HostRef::Address(*ip),
        }
    }

    /// Converts the host to an authority with the port.
    pub fn to_authority(self, port: u16) -> Authority {
        Authority::new(self, port)
    }
}

impl<'a> HostRef<'a> {
    //! Conversions

    /// Converts the host reference to a host.
    pub fn to_host(&self) -> Host {
        match self {
            Self::Name(domain) => Host::Name(domain.to_domain()),
            Self::Address(ip) => Host::Address(*ip),
        }
    }

    /// Converts the host reference to an authority reference with the port.
    pub fn to_authority(&self, port: u16) -> AuthorityRef {
        AuthorityRef::new(*self, port)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, AuthorityRef, Domain, DomainRef, Host, HostRef, IPv4Address};

    #[test]
    fn host_to_ref() {
        let host: Host = Domain::localhost().to_host();
        let result: HostRef = host.to_ref();
        let expected: HostRef = DomainRef::LOCALHOST.to_host();
        assert_eq!(result, expected);

        let host: Host = IPv4Address::LOCALHOST.to_host();
        let result: HostRef = host.to_ref();
        let expected: HostRef = HostRef::Address(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }

    #[test]
    fn host_to_authority() {
        let host: Host = Domain::localhost().to_host();
        let result: Authority = host.to_authority(80);
        let expected: Authority = Authority::new(Domain::localhost().to_host(), 80);
        assert_eq!(result, expected);

        let host: Host = IPv4Address::LOCALHOST.to_host();
        let result: Authority = host.to_authority(80);
        let expected: Authority = Authority::new(IPv4Address::LOCALHOST.to_host(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_host() {
        let host: HostRef = DomainRef::LOCALHOST.to_host();
        let result: Host = host.to_host();
        let expected: Host = Domain::localhost().to_host();
        assert_eq!(result, expected);

        let host: HostRef = IPv4Address::LOCALHOST.to_host_ref();
        let result: Host = host.to_host();
        let expected: Host = Host::Address(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_authority() {
        let host: HostRef = DomainRef::LOCALHOST.to_host();
        let result: AuthorityRef = host.to_authority(80);
        let expected: AuthorityRef = AuthorityRef::new(DomainRef::LOCALHOST.to_host(), 80);
        assert_eq!(result, expected);

        let host: Host = IPv4Address::LOCALHOST.to_host();
        let host: HostRef = host.to_ref();
        let result: AuthorityRef = host.to_authority(80);
        let expected: AuthorityRef = AuthorityRef::new(IPv4Address::LOCALHOST.to_host_ref(), 80);
        assert_eq!(result, expected);
    }
}
