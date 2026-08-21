use crate::{Domain, DomainRef, Endpoint, EndpointRef, Host, HostRef};

impl Domain {
    //! Conversions

    /// Converts the domain to a domain reference.
    pub fn to_domain_ref(&self) -> DomainRef {
        unsafe { DomainRef::new(self.name()) }
    }

    /// Converts the domain to a host.
    pub fn to_host(self) -> Host {
        Host::Name(self)
    }

    /// Converts the domain to an endpoint with the port.
    pub fn to_endpoint(self, port: u16) -> Endpoint {
        Endpoint::new(self, port)
    }
}

impl<'a> DomainRef<'a> {
    //! Conversions

    /// Converts the domain reference to a domain.
    pub fn to_domain(&self) -> Domain {
        unsafe { Domain::new(self.name()) }
    }

    /// Converts the domain reference to a host reference.
    pub fn to_host(&self) -> HostRef {
        HostRef::Name(*self)
    }

    /// Converts the domain reference to an endpoint reference with the port.
    pub fn to_endpoint(&self, port: u16) -> EndpointRef {
        EndpointRef::new(*self, port)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Endpoint, EndpointRef, Host, HostRef};

    #[test]
    fn domain() {
        let domain: Domain = Domain::localhost();
        let result: DomainRef = domain.to_domain_ref();
        assert_eq!(result, DomainRef::LOCALHOST);

        let result: Host = Domain::localhost().to_host();
        let expected: Host = Host::Name(Domain::localhost());
        assert_eq!(result, expected);

        let result: Endpoint = Domain::localhost().to_endpoint(80);
        let expected: Endpoint = Endpoint::new(Domain::localhost(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn domain_ref() {
        let domain: DomainRef = DomainRef::LOCALHOST;

        let result: Domain = domain.to_domain();
        let expected: Domain = Domain::localhost();
        assert_eq!(result, expected);

        let result: HostRef = domain.to_host();
        let expected: HostRef = HostRef::Name(DomainRef::LOCALHOST);
        assert_eq!(result, expected);

        let result: EndpointRef = domain.to_endpoint(80);
        let expected: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);
        assert_eq!(result, expected);
    }
}
