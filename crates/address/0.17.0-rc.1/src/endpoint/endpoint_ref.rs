use crate::DomainRef;

/// A domain reference with an associated port.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
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
        Self {
            domain: tuple.0.into(),
            port: tuple.1,
        }
    }
}

impl<'a> EndpointRef<'a> {
    //! Properties

    /// Gets the domain reference.
    pub const fn domain(&self) -> DomainRef<'a> {
        self.domain
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}

#[cfg(test)]
mod tests {
    use crate::{DomainRef, EndpointRef};

    #[test]
    fn construction() {
        let result: EndpointRef = EndpointRef::new(DomainRef::LOCALHOST, 80);
        assert_eq!(result.domain, DomainRef::LOCALHOST);
        assert_eq!(result.port, 80);

        let result: EndpointRef = (DomainRef::LOCALHOST, 80).into();
        assert_eq!(result.domain, DomainRef::LOCALHOST);
        assert_eq!(result.port, 80);
    }

    #[test]
    fn properties() {
        let endpoint: EndpointRef = (DomainRef::LOCALHOST, 80).into();
        assert_eq!(endpoint.domain(), DomainRef::LOCALHOST);
        assert_eq!(endpoint.port(), 80);
    }
}
