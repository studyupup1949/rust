use crate::{Authority, AuthorityRef, Endpoint, EndpointRef, Host, HostRef};

impl Endpoint {
    //! Conversions

    /// Converts the endpoint to an endpoint reference.
    pub fn to_ref(&self) -> EndpointRef<'_> {
        EndpointRef::new(self.domain(), self.port())
    }

    /// Converts the endpoint to an authority.
    pub fn to_authority(self) -> Authority {
        let (domain, port) = self.into();
        Authority::new(Host::Name(domain), port)
    }
}

impl<'a> EndpointRef<'a> {
    //! Conversions

    /// Converts the endpoint reference to an endpoint.
    pub fn to_endpoint(&self) -> Endpoint {
        Endpoint::new(self.domain().to_domain(), self.port())
    }

    /// Converts the endpoint reference to an authority reference.
    pub fn to_authority(&self) -> AuthorityRef<'_> {
        AuthorityRef::new(HostRef::Name(self.domain()), self.port())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, AuthorityRef, Domain, DomainRef, Endpoint, EndpointRef};

    #[test]
    fn endpoint_to_ref() {
        let endpoint: Endpoint = Endpoint::new(Domain::localhost(), 80);

        let result: EndpointRef = endpoint.to_ref();
        let expected: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn endpoint_to_authority() {
        let endpoint: Endpoint = Endpoint::new(Domain::localhost(), 80);

        let result: Authority = endpoint.to_authority();
        let expected: Authority = Authority::new(Domain::localhost().to_host(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_endpoint() {
        let endpoint: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);

        let result: Endpoint = endpoint.to_endpoint();
        let expected: Endpoint = Endpoint::new(Domain::localhost(), 80);
        assert_eq!(result, expected);
    }

    #[test]
    fn ref_to_authority() {
        let endpoint: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);

        let result: AuthorityRef = endpoint.to_authority();
        let expected: AuthorityRef = AuthorityRef::new(DomainRef::LOCALHOST.to_host(), 80);
        assert_eq!(result, expected);
    }
}
