use std::fmt::{Display, Error, Formatter};

use crate::{Endpoint, Host, SocketAddress};

/// Represents either an endpoint or a socket address.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Authority<'a> {
    /// An Endpoint
    Name(Endpoint<'a>),

    /// A Socket Address
    Address(SocketAddress),
}

impl<'a> Authority<'a> {
    //! Conversions

    /// Converts the authority to an optional endpoint.
    pub fn to_endpoint(&self) -> Option<Endpoint> {
        match self {
            Authority::Name(endpoint) => Some(*endpoint),
            _ => None,
        }
    }

    /// Converts the authority to an optional socket address.
    pub fn to_socket(&self) -> Option<SocketAddress> {
        match self {
            Authority::Address(socket) => Some(*socket),
            _ => None,
        }
    }
}

impl<'a> From<Endpoint<'a>> for Authority<'a> {
    fn from(endpoint: Endpoint<'a>) -> Self {
        Authority::Name(endpoint)
    }
}

impl<'a> From<SocketAddress> for Authority<'a> {
    fn from(socket: SocketAddress) -> Self {
        Authority::Address(socket)
    }
}

impl<'a> Authority<'a> {
    //! Properties

    /// Gets the host.
    pub fn host(&self) -> Host {
        match self {
            Authority::Name(endpoint) => endpoint.host(),
            Authority::Address(socket) => socket.host(),
        }
    }

    /// Gets the port.
    pub fn port(&self) -> u16 {
        match self {
            Authority::Name(endpoint) => endpoint.port(),
            Authority::Address(socket) => socket.port(),
        }
    }
}

impl<'a> Authority<'a> {
    //! Matching

    /// Checks if the authority is an endpoint.
    pub fn is_endpoint(&self) -> bool {
        match self {
            Authority::Name(_) => true,
            _ => false,
        }
    }

    /// Checks if the authority is a socket address.
    pub fn is_socket(&self) -> bool {
        match self {
            Authority::Address(_) => true,
            _ => false,
        }
    }
}

impl<'a> Display for Authority<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Authority::Name(endpoint) => write!(f, "{}", endpoint),
            Authority::Address(socket) => write!(f, "{}", socket),
        }
    }
}

#[cfg(test)]
mod conversion_tests {
    use crate::{Domain, IPv4Address};

    #[test]
    fn to_endpoint() {
        assert_eq!(
            Domain::LOCALHOST.to_authority(80).to_endpoint(),
            Some(Domain::LOCALHOST.to_endpoint(80))
        );
        assert_eq!(IPv4Address::LOCALHOST.to_authority(80).to_endpoint(), None);
    }

    #[test]
    fn to_socket() {
        assert_eq!(Domain::LOCALHOST.to_authority(80).to_socket(), None);
        assert_eq!(
            IPv4Address::LOCALHOST.to_authority(80).to_socket(),
            Some(IPv4Address::LOCALHOST.to_socket(80))
        );
    }
}

#[cfg(test)]
mod from_tests {
    use crate::{Authority, Domain, Endpoint, IPv4Address, SocketAddress};

    #[test]
    fn from_endpoint() {
        let endpoint: Endpoint = Domain::LOCALHOST.to_endpoint(80);
        assert_eq!(Authority::from(endpoint), Authority::Name(endpoint));
    }

    #[test]
    fn from_socket() {
        let socket: SocketAddress = IPv4Address::LOCALHOST.to_socket(80);
        assert_eq!(Authority::from(socket), Authority::Address(socket));
    }
}

#[cfg(test)]
mod property_tests {
    use crate::{Authority, Domain, Host, IPv4Address};

    #[test]
    fn host() {
        let authority: Authority = Domain::LOCALHOST.to_authority(80);
        assert_eq!(authority.host(), Host::Name(Domain::LOCALHOST));

        let authority: Authority = IPv4Address::LOCALHOST.to_authority(80);
        assert_eq!(
            authority.host(),
            Host::Address(IPv4Address::LOCALHOST.to_ip())
        );
    }

    #[test]
    fn port() {
        assert_eq!(IPv4Address::LOCALHOST.to_authority(80).port(), 80);
        assert_eq!(Domain::LOCALHOST.to_authority(80).port(), 80);
    }
}

#[cfg(test)]
mod matching_tests {
    use crate::{Domain, IPv4Address};

    #[test]
    fn is_endpoint() {
        assert_eq!(Domain::LOCALHOST.to_authority(80).is_endpoint(), true);
        assert_eq!(IPv4Address::LOCALHOST.to_authority(80).is_endpoint(), false);
    }

    #[test]
    fn is_socket() {
        assert_eq!(Domain::LOCALHOST.to_authority(80).is_socket(), false);
        assert_eq!(IPv4Address::LOCALHOST.to_authority(80).is_socket(), true);
    }
}

#[cfg(test)]
mod display_tests {
    use crate::{Domain, IPv4Address};

    #[test]
    fn display() {
        assert_eq!(
            Domain::LOCALHOST.to_authority(80).to_string(),
            "localhost:80"
        );
        assert_eq!(
            IPv4Address::LOCALHOST.to_authority(80).to_string(),
            "127.0.0.1:80"
        );
    }
}
