use crate::{Authority, AuthorityRef, Endpoint, EndpointRef};

impl Endpoint {
    //! Conversions

    /// Converts the endpoint to an endpoint reference.
    pub fn to_ref(&self) -> EndpointRef {
        EndpointRef::new(self.domain(), self.port())
    }

    /// Converts the endpoint to an authority.
    pub fn to_authority(self) -> Authority {
        let (domain, port) = self.into();
        Authority::new(domain.to_host(), port)
    }
}

impl<'a> EndpointRef<'a> {
    //! Conversions

    /// Converts the endpoint reference to an endpoint.
    pub fn to_endpoint(&self) -> Endpoint {
        Endpoint::new(self.domain().to_domain(), self.port())
    }

    /// Converts the endpoint reference to an authority reference.
    pub fn to_authority(&self) -> AuthorityRef {
        AuthorityRef::new(self.host(), self.port())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Authority, Domain, DomainRef, Endpoint, EndpointRef};

    #[test]
    fn endpoint() {
        let endpoint: Endpoint = Domain::localhost().to_endpoint(80);

        let result: EndpointRef = endpoint.to_ref();
        let expected: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);
        assert_eq!(result, expected);

        let result: Authority = endpoint.to_authority();
        let expected: Authority = Authority::new(Domain::localhost().to_host(), 80);
        assert_eq!(result, expected);
    }
}
