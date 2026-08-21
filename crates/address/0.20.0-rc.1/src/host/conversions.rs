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
        if let Self::Address(ip) = self { Some(ip) } else { None }
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
        if let Self::Address(ip) = self { Some(ip) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, AuthorityRef, Domain, DomainRef, Host, HostRef, IPAddress, IPv4Address};

    #[test]
    fn host_to_ref() {
        let test_cases: &[(Host, HostRef)] = &[
            (Host::Name(Domain::localhost()), HostRef::Name(DomainRef::LOCALHOST)),
            (
                Host::Address(IPAddress::V4(IPv4Address::LOCALHOST)),
                HostRef::Address(IPAddress::V4(IPv4Address::LOCALHOST)),
            ),
        ];

        for (host, expected) in test_cases {
            let result: HostRef = host.to_ref();
            assert_eq!(result, *expected, "host={:?}", host);
        }
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
        let test_cases: &[(HostRef, Host)] = &[
            (HostRef::Name(DomainRef::LOCALHOST), Host::Name(Domain::localhost())),
            (
                HostRef::Address(IPAddress::V4(IPv4Address::LOCALHOST)),
                Host::Address(IPAddress::V4(IPv4Address::LOCALHOST)),
            ),
        ];

        for (host, expected) in test_cases {
            let result: Host = host.to_host();
            assert_eq!(result, *expected, "host={:?}", host);
        }
    }

    #[test]
    fn ref_to_authority() {
        let test_cases: &[(HostRef, AuthorityRef)] = &[
            (
                DomainRef::LOCALHOST.to_host_ref(),
                AuthorityRef::new(DomainRef::LOCALHOST.to_host_ref(), 80),
            ),
            (
                IPv4Address::LOCALHOST.to_host_ref(),
                AuthorityRef::new(IPv4Address::LOCALHOST.to_host_ref(), 80),
            ),
        ];

        for (host, expected) in test_cases {
            let result: AuthorityRef = host.to_authority_ref(80);
            assert_eq!(result, *expected, "host={:?}", host);
        }
    }

    #[test]
    fn ref_to_domain() {
        let test_cases: &[(HostRef, Option<DomainRef>)] = &[
            (DomainRef::LOCALHOST.to_host_ref(), Some(DomainRef::LOCALHOST)),
            (IPv4Address::LOCALHOST.to_host_ref(), None),
        ];

        for (host, expected) in test_cases {
            let result: Option<DomainRef> = host.to_domain_ref();
            assert_eq!(result, *expected, "host={:?}", host);
        }
    }

    #[test]
    fn ref_to_ip() {
        let test_cases: &[(HostRef, Option<IPAddress>)] = &[
            (
                IPv4Address::LOCALHOST.to_host_ref(),
                Some(IPv4Address::LOCALHOST.to_ip()),
            ),
            (DomainRef::LOCALHOST.to_host_ref(), None),
        ];

        for (host, expected) in test_cases {
            let result: Option<IPAddress> = host.to_ip();
            assert_eq!(result, *expected, "host={:?}", host);
        }
    }
}
