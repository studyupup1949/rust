use std::fmt::{Display, Error, Formatter};

use crate::authority::Authority;
use crate::domain::Domain;

/// Represents a domain name with an associated port.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Endpoint {
    domain: Domain,
    port: u16,
}

impl Endpoint {

    /// Creates a new Endpoint.
    pub fn new(domain: Domain, port: u16) -> Endpoint {
        Endpoint{ domain, port }
    }
}

impl Endpoint {

    /// Gets the domain name.
    pub fn domain(&self) -> &Domain {
        &self.domain
    }

    /// Gets the port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Gets the authority.
    pub fn authority(self) -> Authority {
        Authority::Name(self)
    }
}

impl Display for Endpoint {

    /// ```
    /// use address::endpoint::Endpoint;
    /// use address::domain::Domain;
    ///
    /// let endpoint: Endpoint = Endpoint::new(Domain::get_localhost(), 80);
    ///
    /// assert_eq!(endpoint.to_string(), "localhost:80");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}:{}", self.domain, self.port)
    }
}
