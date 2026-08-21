use crate::Domain;

/// Represents a domain with an associated port.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Endpoint {
    domain: Domain,
    port: u16,
}

impl Endpoint {
    //! Constructors

    /// Creates a new endpoint.
    pub const fn new(domain: Domain, port: u16) -> Self {
        Self { domain, port }
    }
}

impl Endpoint {
    //! Properties

    /// Gets the domain.
    pub const fn domain(&self) -> &Domain {
        &self.domain
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }
}

impl Endpoint {
    //! Deconstructors

    /// Exports the tuple.
    pub fn export(self) -> (Domain, u16) {
        (self.domain, self.port)
    }

    /// Exports the domain.
    pub fn export_domain(self) -> Domain {
        self.domain
    }
}
