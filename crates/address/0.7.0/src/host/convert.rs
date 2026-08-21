use crate::{Domain, DomainRef, Host, HostRef, IPAddress};

impl Host {
    //! Conversions

    /// Converts the host to a host reference.
    pub fn to_host_ref(&self) -> HostRef {
        match self {
            Self::Name(domain) => HostRef::Name(domain.to_domain_ref()),
            Self::Address(ip) => HostRef::Address(*ip),
        }
    }

    /// Converts the host to an optional domain.
    pub fn to_domain(self) -> Option<Domain> {
        match self {
            Self::Name(domain) => Some(domain),
            Self::Address(_) => None,
        }
    }

    /// Converts the host to an optional IP address.
    pub const fn to_ip(&self) -> Option<IPAddress> {
        match self {
            Self::Name(_) => None,
            Self::Address(ip) => Some(*ip),
        }
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

    /// Converts the host reference to an optional domain reference.
    pub const fn to_domain_ref(&self) -> Option<DomainRef> {
        match self {
            Self::Name(domain) => Some(*domain),
            Self::Address(_) => None,
        }
    }

    /// Converts the host reference to an optional IP address.
    pub const fn to_ip(&self) -> Option<IPAddress> {
        match self {
            Self::Name(_) => None,
            Self::Address(ip) => Some(*ip),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Host, HostRef, IPAddress, IPv4Address};

    #[test]
    fn host_to_host_ref() {
        let host: Host = Domain::localhost().into();
        let result: HostRef = host.to_host_ref();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);

        let host: Host = IPv4Address::LOCALHOST.into();
        let result: HostRef = host.to_host_ref();
        let expected: HostRef = HostRef::Address(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }

    #[test]
    fn host_to_domain() {
        let host: Host = Domain::localhost().into();
        let result: Option<Domain> = host.to_domain();
        let expected: Option<Domain> = Some(Domain::localhost());
        assert_eq!(result, expected);

        let host: Host = IPv4Address::LOCALHOST.into();
        let result: Option<Domain> = host.to_domain();
        let expected: Option<Domain> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn host_to_ip() {
        let result: Option<IPAddress> = Host::Name(Domain::localhost()).to_ip();
        let expected: Option<IPAddress> = None;
        assert_eq!(result, expected);

        let result: Option<IPAddress> = Host::Address(IPv4Address::LOCALHOST.to_ip()).to_ip();
        let expected: Option<IPAddress> = Some(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }

    #[test]
    fn host_ref_to_host() {
        let result: Host = HostRef::Name(DomainRef::LOCALHOST).to_host();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);

        let result: Host = HostRef::Address(IPv4Address::LOCALHOST.to_ip()).to_host();
        let expected: Host = Host::Address(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }

    #[test]
    fn host_ref_to_domain_ref() {
        let host: HostRef = HostRef::from(DomainRef::LOCALHOST);
        let result: Option<DomainRef> = host.to_domain_ref();
        let expected: Option<DomainRef> = Some(DomainRef::LOCALHOST);
        assert_eq!(result, expected);

        let host: HostRef = HostRef::from(IPv4Address::LOCALHOST);
        let result: Option<DomainRef> = host.to_domain_ref();
        let expected: Option<DomainRef> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn host_ref_to_ip() {
        let result: Option<IPAddress> = HostRef::Name(DomainRef::LOCALHOST).to_ip();
        let expected: Option<IPAddress> = None;
        assert_eq!(result, expected);

        let result: Option<IPAddress> = HostRef::Address(IPv4Address::LOCALHOST.to_ip()).to_ip();
        let expected: Option<IPAddress> = Some(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }
}
