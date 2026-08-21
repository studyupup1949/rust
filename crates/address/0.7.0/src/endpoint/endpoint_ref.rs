use std::fmt::Debug;

use crate::{DomainRef, HostRef};

/// A domain with an associated port.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct EndpointRef<'a> {
    domain: DomainRef<'a>,
    port: u16,
}

impl<'a> EndpointRef<'a> {
    //! Construction

    /// Creates a new endpoint.
    pub const fn new(domain: DomainRef<'a>, port: u16) -> Self {
        Self { domain, port }
    }
}

impl<'a, D: Into<DomainRef<'a>>> From<(D, u16)> for EndpointRef<'a> {
    fn from(tuple: (D, u16)) -> Self {
        Self::new(tuple.0.into(), tuple.1.into())
    }
}

impl<'a> EndpointRef<'a> {
    //! Properties

    /// Gets the domain.
    pub fn domain(&self) -> DomainRef {
        self.domain
    }

    /// Gets the host.
    pub fn host(&self) -> HostRef {
        HostRef::Name(self.domain)
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
    fn properties() {
        let endpoint: EndpointRef = (DomainRef::LOCALHOST, 80).into();
        assert_eq!(endpoint.domain(), DomainRef::LOCALHOST);
        assert_eq!(endpoint.port(), 80);
    }
}
