use std::fmt::{Display, Error, Formatter};

use crate::{Authority, Domain, Host};

/// Represents a domain with an associated port.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Endpoint<'a> {
    domain: Domain<'a>,
    port: u16,
}

impl<'a> Endpoint<'a> {
    //! Constructors

    /// Creates a new endpoint.
    pub const fn new(domain: Domain<'a>, port: u16) -> Endpoint<'a> {
        Self { domain, port }
    }
}

impl<'a> Endpoint<'a> {
    //! Conversions

    /// Converts the endpoint to an authority.
    pub const fn to_authority(&self) -> Authority {
        Authority::Name(*self)
    }
}

impl<'a, D: Into<Domain<'a>>> From<(D, u16)> for Endpoint<'a> {
    fn from(tuple: (D, u16)) -> Self {
        Endpoint {
            domain: tuple.0.into(),
            port: tuple.1,
        }
    }
}

impl<'a> Endpoint<'a> {
    //! Properties

    /// Gets the domain.
    pub const fn domain(&self) -> Domain {
        self.domain
    }

    /// Gets the port.
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Gets the host.
    pub const fn host(&self) -> Host {
        self.domain.to_host()
    }
}

impl<'a> Display for Endpoint<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}:{}", self.domain, self.port)
    }
}

#[cfg(test)]
mod constructor_tests {
    use crate::{Domain, Endpoint};

    #[test]
    fn new() {
        let endpoint: Endpoint = Endpoint::new(Domain::LOCALHOST, 80);
        assert_eq!(endpoint.domain, Domain::LOCALHOST);
        assert_eq!(endpoint.port, 80);
    }
}

#[cfg(test)]
mod conversion_tests {
    use crate::{Authority, Domain};

    #[test]
    fn to_endpoint() {
        assert_eq!(
            Domain::LOCALHOST.to_endpoint(80).to_authority(),
            Authority::Name(Domain::LOCALHOST.to_endpoint(80))
        );
    }
}

#[cfg(test)]
mod from_tests {
    use crate::{Domain, Endpoint};

    #[test]
    fn from_tuple() {
        let endpoint: Endpoint = (Domain::LOCALHOST, 80).into();
        assert_eq!(endpoint, Endpoint::new(Domain::LOCALHOST, 80));
    }
}

#[cfg(test)]
mod property_tests {
    use crate::{Domain, Endpoint};

    #[test]
    fn domain() {
        assert_eq!(
            Endpoint::new(Domain::LOCALHOST, 80).domain(),
            Domain::LOCALHOST
        );
    }

    #[test]
    fn port() {
        assert_eq!(Endpoint::new(Domain::LOCALHOST, 80).port(), 80);
    }

    #[test]
    fn host() {
        assert_eq!(
            Endpoint::new(Domain::LOCALHOST, 80).host(),
            Domain::LOCALHOST.to_host()
        )
    }
}

#[cfg(test)]
mod display_tests {
    use crate::{Domain, Endpoint};

    #[test]
    fn display() {
        assert_eq!(
            Endpoint::new(Domain::LOCALHOST, 80).to_string(),
            "localhost:80"
        );
    }
}
