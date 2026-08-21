use crate::{Domain, DomainRef, Endpoint, EndpointRef, Host, HostRef};

impl Domain {
    //! Conversions

    /// Converts the domain to a domain reference.
    pub fn to_ref(&self) -> DomainRef<'_> {
        unsafe { DomainRef::new(self.name()) }
    }

    /// Converts the domain to an endpoint with the `port`.
    pub fn to_endpoint(self, port: u16) -> Endpoint {
        Endpoint::new(self, port)
    }

    /// Converts the domain to a host.
    pub fn to_host(self) -> Host {
        Host::Name(self)
    }
}

impl<'a> DomainRef<'a> {
    //! Conversions

    /// Converts the domain reference to a domain.
    pub fn to_domain(&self) -> Domain {
        unsafe { Domain::new(self.name()) }
    }

    /// Converts the domain reference to an endpoint reference with the `port`.
    pub const fn to_endpoint(&self, port: u16) -> EndpointRef<'_> {
        EndpointRef::new(*self, port)
    }

    /// Converts the domain reference to a host reference.
    pub const fn to_host(&self) -> HostRef<'_> {
        HostRef::Name(*self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Endpoint, EndpointRef, Host, HostRef};

    #[test]
    fn domain_to_ref() {
        let domain: Domain = Domain::localhost();

        let result: DomainRef = domain.to_ref();
        let expected: DomainRef = DomainRef::LOCALHOST;
        assert_eq!(result, expected);
    }

    #[test]
    fn domain_to_endpoint() {
        let domain: Domain = Domain::localhost();

        let result: Endpoint = domain.to_endpoint(80);
        let expected: Endpoint = Endpoint::new(Domain::localhost(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn domain_to_host() {
        let domain: Domain = Domain::localhost();

        let result: Host = domain.to_host();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_domain() {
        let domain: DomainRef = DomainRef::LOCALHOST;

        let result: Domain = domain.to_domain();
        let expected: Domain = Domain::localhost();
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_endpoint() {
        let domain: DomainRef = DomainRef::LOCALHOST;

        let result: EndpointRef = domain.to_endpoint(80);
        let expected: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_host() {
        let domain: DomainRef = DomainRef::LOCALHOST;

        let result: HostRef = domain.to_host();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);
    }
}
