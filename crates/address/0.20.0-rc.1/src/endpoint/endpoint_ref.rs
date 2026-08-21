use crate::{DomainRef, Endpoint};

/// A domain reference with an associated port.
#[must_use]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct EndpointRef<'a> {
    domain: DomainRef<'a>,
    port: u16,
}

impl<'a> EndpointRef<'a> {
    //! Construction

    /// Creates a new endpoint reference.
    pub const fn new(domain: DomainRef<'a>, port: u16) -> Self {
        Self { domain, port }
    }
}

impl<'a, D: Into<DomainRef<'a>>> From<(D, u16)> for EndpointRef<'a> {
    fn from(tuple: (D, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1)
    }
}

impl<'a> From<EndpointRef<'a>> for (DomainRef<'a>, u16) {
    fn from(endpoint: EndpointRef<'a>) -> Self {
        (endpoint.domain, endpoint.port)
    }
}

impl<'a> From<&'a Endpoint> for EndpointRef<'a> {
    fn from(endpoint: &'a Endpoint) -> Self {
        endpoint.to_ref()
    }
}

impl<'a> PartialEq<Endpoint> for EndpointRef<'a> {
    fn eq(&self, other: &Endpoint) -> bool {
        *self == other.to_ref()
    }
}

impl<'a> EndpointRef<'a> {
    //! Properties

    /// Gets the domain reference.
    pub const fn domain(self) -> DomainRef<'a> {
        self.domain
    }

    /// Gets the port.
    #[must_use]
    pub const fn port(self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Endpoint, EndpointRef};

    #[test]
    fn construction() {
        let result: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);
        assert_eq!(result.domain, DomainRef::LOCALHOST);
        assert_eq!(result.port, 80);

        let result: EndpointRef = (DomainRef::LOCALHOST, 80).into();
        assert_eq!(result.domain, DomainRef::LOCALHOST);
        assert_eq!(result.port, 80);

        let owned: Endpoint = Endpoint::new(Domain::localhost(), 80);
        let result: EndpointRef = (&owned).into();
        assert_eq!(result.domain, DomainRef::LOCALHOST);
        assert_eq!(result.port, 80);
    }

    #[test]
    fn deconstruction() {
        let endpoint: EndpointRef = (DomainRef::LOCALHOST, 80).into();
        let (domain, port): (DomainRef, u16) = endpoint.into();
        assert_eq!(domain, DomainRef::LOCALHOST);
        assert_eq!(port, 80);
    }

    #[test]
    fn equality() {
        let owned: Endpoint = Endpoint::new(Domain::localhost(), 80);
        assert_eq!(EndpointRef::new(DomainRef::LOCALHOST, 80), owned);
        assert_ne!(EndpointRef::new(DomainRef::LOCALHOST, 81), owned);
    }

    #[test]
    fn properties() {
        let endpoint: EndpointRef = (DomainRef::LOCALHOST, 80).into();
        assert_eq!(endpoint.domain(), DomainRef::LOCALHOST);
        assert_eq!(endpoint.port(), 80);
    }
}
