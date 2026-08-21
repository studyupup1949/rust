use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use crate::{IPAddress, IPv4Address, IPv6Address};

impl FromStr for IPv4Address {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ipv4Addr::from_str(s).map_err(|_| ())?.into())
    }
}

impl FromStr for IPv6Address {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ipv6Addr::from_str(s).map_err(|_| ())?.into())
    }
}

impl FromStr for IPAddress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ip) = IPv4Address::from_str(s) {
            Ok(ip.to_ip())
        } else if let Ok(ip) = IPv6Address::from_str(s) {
            Ok(ip.to_ip())
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn v4() {
        let result: Result<IPv4Address, ()> = IPv4Address::from_str("127.0.0.1");
        assert_eq!(result, Ok(IPv4Address::LOCALHOST));
    }

    #[test]
    fn v6() {
        let result: Result<IPv6Address, ()> = IPv6Address::from_str("::1");
        assert_eq!(result, Ok(IPv6Address::LOCALHOST));
    }

    #[test]
    fn ip() {
        let result: Result<IPAddress, ()> = IPAddress::from_str("127.0.0.1");
        assert_eq!(result, Ok(IPv4Address::LOCALHOST.to_ip()));

        let result: Result<IPAddress, ()> = IPAddress::from_str("::1");
        assert_eq!(result, Ok(IPv6Address::LOCALHOST.to_ip()));
    }
}
