use crate::domain::Domain;

/// Represents a domain name with an associated port.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Endpoint {
    domain: Domain,
    port: u16,
}

impl Endpoint {

    /// Creates a new Endpoint.
    ///
    /// ```
    /// use address::endpoint::Endpoint;
    /// use address::domain::Domain;
    /// use std::convert::TryInto;
    ///
    /// let domain: Domain = "localhost".try_into().ok().unwrap();
    /// let endpoint: Endpoint = Endpoint::new(domain.clone(), 80);
    /// assert_eq!(endpoint.domain(), &domain);
    /// assert_eq!(endpoint.port(), 80);
    /// ```
    pub fn new(domain: Domain, port: u16) -> Endpoint {
        Endpoint{ domain, port }
    }
}

impl Endpoint {

    /// Gets the domain.
    pub fn domain(&self) -> &Domain {
        &self.domain
    }

    /// Gets the port.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl ToString for Endpoint {

    /// ```
    /// use address::endpoint::Endpoint;
    /// use address::domain::Domain;
    /// use std::convert::TryInto;
    ///
    /// let domain: Domain = "localhost".try_into().ok().unwrap();
    /// let endpoint: Endpoint = Endpoint::new(domain.clone(), 80);
    /// assert_eq!(endpoint.to_string(), "localhost:80")
    /// ```
    fn to_string(&self) -> String {
        format!("{}:{}", self.domain.to_string(), self.port)
    }
}
