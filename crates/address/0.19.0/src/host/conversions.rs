use crate::{Authority, AuthorityRef, Domain, DomainRef, Host, HostRef, IPAddress};

impl Host {
    //! Conversions

    /// Converts the host to a host reference.
    pub fn to_ref(&self) -> HostRef<'_> {
        match self {
            Self::Name(domain) => HostRef::Name(domain.to_ref()),
            Self::Address(ip) => HostRef::Address(*ip),
        }
    }

    /// Converts the host to an authority with the `port`.
    pub const fn to_authority(self, port: u16) -> Authority {
        Authority::new(self, port)
    }

    /// Converts the host to an optional domain.
    #[must_use]
    pub fn to_domain(self) -> Option<Domain> {
        if let Self::Name(domain) = self {
            Some(domain)
        } else {
            None
        }
    }

    /// Converts the host to an optional IP address.
    #[must_use]
    pub fn to_ip(self) -> Option<IPAddress> {
        if let Self::Address(ip) = self {
            Some(ip)
        } else {
            None
        }
    }
}

impl<'a> HostRef<'a> {
    //! Conversions

    /// Converts the host reference to a host.
    pub fn to_host(self) -> Host {
        match self {
            Self::Name(domain) => Host::Name(domain.to_domain()),
            Self::Address(ip) => Host::Address(ip),
        }
    }

    /// Converts the host reference to an authority reference with the `port`.
    pub const fn to_authority_ref(self, port: u16) -> AuthorityRef<'a> {
        AuthorityRef::new(self, port)
    }

    /// Converts the host reference to an optional domain reference.
    #[must_use]
    pub const fn to_domain_ref(self) -> Option<DomainRef<'a>> {
        if let Self::Name(domain) = self {
            Some(domain)
        } else {
            None
        }
    }

    /// Converts the host reference to an optional IP address.
    #[must_use]
    pub const fn to_ip(self) -> Option<IPAddress> {
        if let Self::Address(ip) = self {
            Some(ip)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Authority, AuthorityRef, Domain, DomainRef, Host, HostRef, IPAddress, IPv4Address,
    };

    #[test]
    fn host_to_ref() {
        let host: Host = Host::Name(Domain::localhost());
        let result: HostRef = host.to_ref();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);

        let host: Host = Host::Address(IPAddress::V4(IPv4Address::LOCALHOST));
        let result: HostRef = host.to_ref();
        let expected: HostRef = HostRef::Address(IPAddress::V4(IPv4Address::LOCALHOST));
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
    fn host_to_domain() {
        let host: Host = Domain::localhost().to_host();
        let result: Option<Domain> = host.to_domain();
        let expected: Option<Domain> = Some(Domain::localhost());
        assert_eq!(result, expected);

        let host: Host = IPv4Address::LOCALHOST.to_host();
        let result: Option<Domain> = host.to_domain();
        let expected: Option<Domain> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn host_to_ip() {
        let host: Host = IPv4Address::LOCALHOST.to_host();
        let result: Option<IPAddress> = host.to_ip();
        let expected: Option<IPAddress> = Some(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);

        let host: Host = Domain::localhost().to_host();
        let result: Option<IPAddress> = host.to_ip();
        let expected: Option<IPAddress> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_host() {
        let host: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        let result: Host = host.to_host();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);

        let host: HostRef = HostRef::Address(IPAddress::V4(IPv4Address::LOCALHOST));
        let result: Host = host.to_host();
        let expected: Host = Host::Address(IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_authority() {
        let host: HostRef = DomainRef::LOCALHOST.to_host_ref();
        let result: AuthorityRef = host.to_authority_ref(80);
        let expected: AuthorityRef = AuthorityRef::new(DomainRef::LOCALHOST.to_host_ref(), 80);
        assert_eq!(result, expected);

        let host: Host = IPv4Address::LOCALHOST.to_host();
        let host: HostRef = host.to_ref();
        let result: AuthorityRef = host.to_authority_ref(80);
        let expected: AuthorityRef = AuthorityRef::new(IPv4Address::LOCALHOST.to_host_ref(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_domain() {
        let host: HostRef = DomainRef::LOCALHOST.to_host_ref();
        let result: Option<DomainRef> = host.to_domain_ref();
        let expected: Option<DomainRef> = Some(DomainRef::LOCALHOST);
        assert_eq!(result, expected);

        let host: HostRef = IPv4Address::LOCALHOST.to_host_ref();
        let result: Option<DomainRef> = host.to_domain_ref();
        let expected: Option<DomainRef> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_ip() {
        let host: HostRef = IPv4Address::LOCALHOST.to_host_ref();
        let result: Option<IPAddress> = host.to_ip();
        let expected: Option<IPAddress> = Some(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);

        let host: HostRef = DomainRef::LOCALHOST.to_host_ref();
        let result: Option<IPAddress> = host.to_ip();
        let expected: Option<IPAddress> = None;
        assert_eq!(result, expected);
    }
}
