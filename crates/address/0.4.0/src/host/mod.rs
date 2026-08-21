use std::fmt::{Display, Error, Formatter};

use crate::domain::Domain;
use crate::ip::IPAddress;

/// Represents an IP address or a domain name.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Host {

    /// An Address
    Address(IPAddress),

    /// A Name
    Name(Domain),
}

impl Display for Host {

    /// ```
    /// use address::ip::IPv4Address;
    /// use address::domain::Domain;
    ///
    /// assert_eq!(IPv4Address::LOCALHOST.ip().host().to_string(), "127.0.0.1");
    /// assert_eq!(Domain::get_localhost().host().to_string(), "localhost");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Host::Address(ip) => write!(f, "{}", ip),
            Host::Name(name) => write!(f, "{}", name),
        }
    }
}
