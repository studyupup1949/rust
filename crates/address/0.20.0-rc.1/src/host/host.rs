use crate::{Domain, DomainRef, HostRef, IPAddress};

/// Either a domain or an IP address.
#[must_use]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Host {
    /// A domain.
    Name(Domain),

    /// An IP address.
    Address(IPAddress),
}

impl From<Domain> for Host {
    fn from(domain: Domain) -> Self {
        Self::Name(domain)
    }
}

impl<'a> From<DomainRef<'a>> for Host {
    fn from(domain: DomainRef<'a>) -> Self {
        Self::Name(domain.to_domain())
    }
}

impl<A: Into<IPAddress>> From<A> for Host {
    fn from(ip: A) -> Self {
        Self::Address(ip.into())
    }
}

impl<'a> From<HostRef<'a>> for Host {
    fn from(host: HostRef<'a>) -> Self {
        host.to_host()
    }
}

impl<'a> PartialEq<HostRef<'a>> for Host {
    fn eq(&self, other: &HostRef<'a>) -> bool {
        self.to_ref() == *other
    }
}

impl Host {
    //! Matching

    /// Checks if the host is a domain.
    #[must_use]
    pub const fn is_domain(&self) -> bool {
        matches!(self, Self::Name(_))
    }

    /// Checks if the host is an IP address.
    #[must_use]
    pub const fn is_ip(&self) -> bool {
        matches!(self, Self::Address(_))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Host, HostRef, IPAddress, IPv4Address};

    #[test]
    fn construction() {
        let result: Host = Domain::localhost().into();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);

        let result: Host = IPv4Address::LOCALHOST.into();
        let expected: Host = Host::Address(IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(result, expected);

        let result: Host = DomainRef::LOCALHOST.into();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);

        let host: HostRef = DomainRef::LOCALHOST.to_host_ref();
        let result: Host = host.into();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);
    }

    #[test]
    fn equality() {
        let host: Host = Domain::localhost().into();
        assert_eq!(host, HostRef::Name(DomainRef::LOCALHOST));
        assert_ne!(host, IPv4Address::LOCALHOST.to_host_ref());
    }

    #[test]
    fn matching() {
        let test_cases: &[(Host, bool, bool)] = &[
            (Domain::localhost().into(), true, false),
            (IPv4Address::LOCALHOST.into(), false, true),
        ];

        for (host, is_domain, is_ip) in test_cases {
            assert_eq!(host.is_domain(), *is_domain, "host={:?}", host);
            assert_eq!(host.is_ip(), *is_ip, "host={:?}", host);
        }
    }
}
