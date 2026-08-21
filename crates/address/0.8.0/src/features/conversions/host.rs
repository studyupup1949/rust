use crate::{Authority, AuthorityRef, Domain, DomainRef, Host, HostRef, IPAddress};

impl Host {
    /// Converts the host to a host reference.
    pub fn to_ref(&self) -> HostRef {
        match self {
            Self::Name(domain) => HostRef::Name(domain.to_ref()),
            Self::Address(ip) => HostRef::Address(*ip),
        }
    }

    /// Converts the host to an optional domain.
    pub fn to_domain(self) -> Option<Domain> {
        if let Self::Name(domain) = self {
            Some(domain)
        } else {
            None
        }
    }

    /// Converts the host to an optional IP address.
    pub fn to_ip(&self) -> Option<IPAddress> {
        if let Self::Address(ip) = self {
            Some(*ip)
        } else {
            None
        }
    }

    /// Converts the host to an authority with the port.
    pub fn to_authority(self, port: u16) -> Authority {
        Authority::new(self, port)
    }
}

impl<'a> HostRef<'a> {
    /// Converts the host reference to a host.
    pub fn to_host(&self) -> Host {
        match self {
            Self::Name(domain) => domain.to_domain().to_host(),
            Self::Address(ip) => ip.to_host(),
        }
    }

    /// Converts the host reference to an optional domain reference.
    pub fn to_domain(&self) -> Option<DomainRef> {
        if let Self::Name(domain) = self {
            Some(*domain)
        } else {
            None
        }
    }

    /// Converts the host reference to an optional IP address.
    pub fn to_ip(&self) -> Option<IPAddress> {
        if let Self::Address(ip) = self {
            Some(*ip)
        } else {
            None
        }
    }

    /// Converts the host reference to an authority reference with the port.
    pub fn to_authority(&self, port: u16) -> AuthorityRef {
        AuthorityRef::new(*self, port)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Authority, AuthorityRef, Domain, DomainRef, Host, HostRef, IPAddress, IPv4Address,
    };

    #[test]
    fn host_to_ref() {
        let host: Host = Domain::localhost().to_host();
        let result: HostRef = host.to_ref();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
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
        let host: Host = Domain::localhost().to_host();
        let result: Option<IPAddress> = host.to_ip();
        let expected: Option<IPAddress> = None;
        assert_eq!(result, expected);

        let host: Host = IPv4Address::LOCALHOST.to_host();
        let result: Option<IPAddress> = host.to_ip();
        let expected: Option<IPAddress> = Some(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }

    #[test]
    fn host_to_authority() {
        let host: Host = Domain::localhost().to_host();
        let result: Authority = host.to_authority(80);
        let expected: Authority = Authority::new(Domain::localhost(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_host() {
        let host: HostRef = DomainRef::LOCALHOST.to_host();
        let result: Host = host.to_host();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_domain() {
        let host: HostRef = DomainRef::LOCALHOST.to_host();
        let result: Option<DomainRef> = host.to_domain();
        let expected: Option<DomainRef> = Some(DomainRef::LOCALHOST);
        assert_eq!(result, expected);

        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();
        let host: HostRef = HostRef::Address(ip);
        let result: Option<DomainRef> = host.to_domain();
        let expected: Option<DomainRef> = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_ip() {
        let host: HostRef = DomainRef::LOCALHOST.to_host();
        let result: Option<IPAddress> = host.to_ip();
        let expected: Option<IPAddress> = None;
        assert_eq!(result, expected);

        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();
        let host: HostRef = HostRef::Address(ip);
        let result: Option<IPAddress> = host.to_ip();
        let expected: Option<IPAddress> = Some(IPv4Address::LOCALHOST.to_ip());
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_authority() {
        let host: HostRef = DomainRef::LOCALHOST.to_host();
        let result: AuthorityRef = host.to_authority(80);
        let expected: AuthorityRef = AuthorityRef::new(DomainRef::LOCALHOST.to_host(), 80);
        assert_eq!(result, expected);
    }
}
