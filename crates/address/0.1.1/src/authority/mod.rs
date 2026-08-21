use crate::host::Host;
use crate::ip::IPAddress;

/// Represents a host with an associated port.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Authority {
    host: Host,
    port: u16,
}

impl Authority {

    /// Creates a new Authority.
    ///
    /// ```
    /// use address::authority::Authority;
    /// use address::host::Host;
    /// use std::convert::TryInto;
    /// use address::ip::IPv4Address;
    ///
    /// let host: Host = Host::Address(IPv4Address::LOCALHOST.ip());
    /// let authority: Authority = Authority::new(host.clone(), 80);
    /// assert_eq!(authority.host(), &host);
    /// assert_eq!(authority.port(), 80);
    /// ```
    pub fn new(host: Host, port: u16) -> Authority {
        Authority{ host, port }
    }
}

impl Authority {

    /// Gets the host.
    pub fn host(&self) -> &Host {
        &self.host
    }

    /// Gets the port.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl ToString for Authority {

    /// ```
    /// use address::authority::Authority;
    /// use address::host::Host;
    /// use address::ip::{IPv4Address, IPv6Address};
    ///
    /// let host: Host = Host::Address(IPv4Address::LOCALHOST.ip());
    /// let authority: Authority = Authority::new(host.clone(), 80);
    /// assert_eq!(authority.to_string(), "127.0.0.1:80");
    ///
    /// let host: Host = Host::Address(IPv6Address::LOCALHOST.ip());
    /// let authority: Authority = Authority::new(host.clone(), 80);
    /// assert_eq!(authority.to_string(), "[::1]:80");
    /// ```
    fn to_string(&self) -> String {
        let brackets: bool = match &self.host {
            Host::Name(_domain) => false,
            Host::Address(ip) => {
                match ip {
                    IPAddress::V4(_ip4) => false,
                    IPAddress::V6(_ip6) => true,
                }
            },
        };
        if brackets {
            format!("[{}]:{}", self.host.to_string(), self.port)
        } else {
            format!("{}:{}", self.host.to_string(), self.port)
        }
    }
}
