use crate::domain::Domain;
use crate::ip::IPAddress;

/// Represents either an IP address or a domain name.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Host {

    /// An IP Address
    Address(IPAddress),

    /// A Domain Name
    Name(Domain),
}

impl ToString for Host {

    /// ```
    /// use address::domain::Domain;
    /// use address::host::Host;
    /// use address::ip::IPv4Address;
    /// use std::convert::TryFrom;
    ///
    /// assert_eq!(Host::Address(IPv4Address::LOCALHOST.ip()).to_string(), "127.0.0.1");
    /// let domain: Domain = Domain::try_from("localhost").ok().unwrap();
    /// assert_eq!(Host::Name(domain).to_string(), "localhost");
    /// ```
    fn to_string(&self) -> String {
        match self {
            Host::Address(ip) => ip.to_string(),
            Host::Name(domain) => domain.to_string(),
        }
    }
}
