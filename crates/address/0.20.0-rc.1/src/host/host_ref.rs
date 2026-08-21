use crate::{DomainRef, Host, IPAddress};

/// Either a domain reference or an IP address.
#[must_use]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum HostRef<'a> {
    /// A domain reference.
    Name(DomainRef<'a>),

    /// An IP address.
    Address(IPAddress),
}

impl<'a> From<DomainRef<'a>> for HostRef<'a> {
    fn from(domain: DomainRef<'a>) -> Self {
        Self::Name(domain)
    }
}

impl<'a, A: Into<IPAddress>> From<A> for HostRef<'a> {
    fn from(ip: A) -> Self {
        Self::Address(ip.into())
    }
}

impl<'a> From<&'a Host> for HostRef<'a> {
    fn from(host: &'a Host) -> Self {
        host.to_ref()
    }
}

impl<'a> PartialEq<Host> for HostRef<'a> {
    fn eq(&self, other: &Host) -> bool {
        *self == other.to_ref()
    }
}

impl<'a> HostRef<'a> {
    //! Matching

    /// Checks if the host is a domain.
    #[must_use]
    pub const fn is_domain(self) -> bool {
        matches!(self, Self::Name(_))
    }

    /// Checks if the host is an IP address.
    #[must_use]
    pub const fn is_ip(self) -> bool {
        matches!(self, Self::Address(_))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Host, HostRef, IPAddress, IPv4Address};

    #[test]
    fn construction() {
        let result: HostRef = DomainRef::LOCALHOST.into();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);

        let result: HostRef = IPAddress::V4(IPv4Address::LOCALHOST).into();
        let expected: HostRef = HostRef::Address(IPAddress::V4(IPv4Address::LOCALHOST));
        assert_eq!(result, expected);

        let owned: Host = Domain::localhost().into();
        let result: HostRef = (&owned).into();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);
    }

    #[test]
    fn equality() {
        let owned: Host = Domain::localhost().into();
        let host: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(host, owned);
        assert_ne!(IPv4Address::LOCALHOST.to_host_ref(), owned);
    }

    #[test]
    fn matching() {
        let test_cases: &[(HostRef, bool, bool)] = &[
            (DomainRef::LOCALHOST.into(), true, false),
            (IPv4Address::LOCALHOST.into(), false, true),
        ];

        for (host, is_domain, is_ip) in test_cases {
            assert_eq!(host.is_domain(), *is_domain, "host={:?}", host);
            assert_eq!(host.is_ip(), *is_ip, "host={:?}", host);
        }
    }
}
