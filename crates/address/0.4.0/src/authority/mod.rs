use std::fmt::{Display, Error, Formatter};

use crate::endpoint::Endpoint;
use crate::socket::SocketAddress;

/// Represents a socket address or an endpoint.
pub enum Authority {

    /// A SocketAddress
    Address(SocketAddress),

    // An Endpoint
    Name(Endpoint)
}

impl Display for Authority {

    /// ```
    ///
    /// use address::authority::Authority;
    /// use address::socket::SocketAddress;
    /// use address::ip::IPv4Address;
    /// use address::endpoint::Endpoint;
    /// use address::domain::Domain;
    ///
    /// let authority: Authority = SocketAddress::new(IPv4Address::LOCALHOST.ip(), 80).authority();
    /// assert_eq!(authority.to_string(), "127.0.0.1:80");
    ///
    /// let authority: Authority = Endpoint::new(Domain::get_localhost(), 443).authority();
    /// assert_eq!(authority.to_string(), "localhost:443");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Authority::Address(socket) => write!(f, "{}", socket),
            Authority::Name(endpoint) => write!(f, "{}", endpoint),
        }
    }
}
