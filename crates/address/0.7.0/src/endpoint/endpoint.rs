use std::fmt::Debug;

use crate::{Domain, DomainRef, HostRef};

/// A domain with an associated port.
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
        Self::new(tuple.0.into(), tuple.1.into())
    }
}

impl Endpoint {
    //! Properties

    /// Gets the domain.
    pub fn domain(&self) -> DomainRef {
        self.domain.to_domain_ref()
    }

    /// Gets the host.
    pub fn host(&self) -> HostRef {
        HostRef::Name(self.domain.to_domain_ref())
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}

impl From<Endpoint> for (Domain, u16) {
    fn from(endpoint: Endpoint) -> Self {
        (endpoint.domain, endpoint.port)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, DomainRef, Endpoint};

    #[test]
    fn properties() {
        let endpoint: Endpoint = (Domain::localhost(), 80).into();
        assert_eq!(endpoint.domain(), DomainRef::LOCALHOST);
        assert_eq!(endpoint.port(), 80);
    }
}
