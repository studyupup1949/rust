use crate::{Domain, DomainRef, EndpointRef};

/// A domain with an associated port.
#[must_use]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Endpoint {
    domain: Domain,
    port: u16,
}

impl Endpoint {
    //! Construction

    /// Creates a new endpoint.
    pub const fn new(domain: Domain, port: u16) -> Self {
        Self { domain, port }
    }
}

impl<D: Into<Domain>> From<(D, u16)> for Endpoint {
    fn from(tuple: (D, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1)
    }
}

impl From<Endpoint> for (Domain, u16) {
    fn from(endpoint: Endpoint) -> Self {
        (endpoint.domain, endpoint.port)
    }
}

impl<'a> From<EndpointRef<'a>> for Endpoint {
    fn from(endpoint: EndpointRef<'a>) -> Self {
        endpoint.to_endpoint()
    }
}

impl<'a> PartialEq<EndpointRef<'a>> for Endpoint {
    fn eq(&self, other: &EndpointRef<'a>) -> bool {
        self.to_ref() == *other
    }
}

impl Endpoint {
    //! Properties

    /// Gets the domain reference.
    pub fn domain(&self) -> DomainRef<'_> {
        self.domain.to_ref()
    }

    /// Gets the port.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Endpoint, EndpointRef};

    #[test]
    fn construction() {
        let endpoint: Endpoint = Endpoint::new(Domain::localhost(), 80);
        assert_eq!(endpoint.domain, Domain::localhost());
        assert_eq!(endpoint.port, 80);

        let endpoint: Endpoint = (DomainRef::LOCALHOST, 80).into();
        assert_eq!(endpoint.domain, Domain::localhost());
        assert_eq!(endpoint.port, 80);

        let endpoint: Endpoint = EndpointRef::new(DomainRef::LOCALHOST, 80).into();
        assert_eq!(endpoint.domain, Domain::localhost());
        assert_eq!(endpoint.port, 80);
    }

    #[test]
    fn deconstruction() {
        let endpoint: Endpoint = Endpoint::new(Domain::localhost(), 80);

        let (domain, port) = endpoint.into();
        assert_eq!(domain, Domain::localhost());
        assert_eq!(port, 80);
    }

    #[test]
    fn equality() {
        let endpoint: Endpoint = Endpoint::new(Domain::localhost(), 80);
        assert_eq!(endpoint, EndpointRef::new(DomainRef::LOCALHOST, 80));
        assert_ne!(endpoint, EndpointRef::new(DomainRef::LOCALHOST, 81));
    }

    #[test]
    fn properties() {
        let endpoint: Endpoint = (Domain::localhost(), 80).into();
        assert_eq!(endpoint.domain(), DomainRef::LOCALHOST);
        assert_eq!(endpoint.port(), 80);
    }
}
