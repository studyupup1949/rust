use crate::{Authority, AuthorityRef, Endpoint, EndpointRef, Host, HostRef};

impl Endpoint {
    //! Conversions

    /// Converts the endpoint to an endpoint reference.
    #[must_use]
    pub fn to_ref(&self) -> EndpointRef<'_> {
        EndpointRef::new(self.domain(), self.port())
    }

    /// Converts the endpoint to an authority.
    #[must_use]
    pub fn to_authority(self) -> Authority {
        let (domain, port) = self.into();
        Authority::new(Host::Name(domain), port)
    }
}

impl<'a> EndpointRef<'a> {
    //! Conversions

    /// Converts the endpoint reference to an endpoint.
    #[must_use]
    pub fn to_endpoint(self) -> Endpoint {
        Endpoint::new(self.domain().to_domain(), self.port())
    }

    /// Converts the endpoint reference to an authority reference.
    #[must_use]
    pub const fn to_authority_ref(self) -> AuthorityRef<'a> {
        AuthorityRef::new(HostRef::Name(self.domain()), self.port())
    }
}

impl<'a> From<&'a Endpoint> for EndpointRef<'a> {
    fn from(endpoint: &'a Endpoint) -> Self {
        endpoint.to_ref()
    }
}

impl<'a> PartialEq<EndpointRef<'a>> for Endpoint {
    fn eq(&self, other: &EndpointRef<'a>) -> bool {
        self.to_ref() == *other
    }
}

impl<'a> PartialEq<Endpoint> for EndpointRef<'a> {
    fn eq(&self, other: &Endpoint) -> bool {
        *self == other.to_ref()
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

        let result: AuthorityRef = endpoint.to_authority_ref();
        let expected: AuthorityRef = AuthorityRef::new(DomainRef::LOCALHOST.to_host_ref(), 80);
        assert_eq!(result, expected);
    }
}
