use std::str::FromStr;

use crate::{IPv4Address, IPv6Address, SocketAddress, SocketAddressV4, SocketAddressV6};

impl FromStr for SocketAddressV4 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = crate::parse::parse_port(s)?;
        if let Ok(v4) = IPv4Address::from_str(s) {
            Ok(SocketAddressV4::new(v4, port))
        } else {
            Err(())
        }
    }
}

impl FromStr for SocketAddressV6 {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = crate::parse::parse_port(s)?;
        if s.is_empty() || s.as_bytes()[0] != b'[' || s.as_bytes()[s.len() - 1] != b']' {
            Err(())
        } else {
            let s: &str = &s[1..(s.len() - 1)];
            if let Ok(v6) = IPv6Address::from_str(s) {
                Ok(SocketAddressV6::new(v6, port))
            } else {
                Err(())
            }
        }
    }
}

impl FromStr for SocketAddress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, port) = crate::parse::parse_port(s)?;
        if let Ok(v4) = IPv4Address::from_str(s) {
            Ok(SocketAddress::new(v4.to_ip(), port))
        } else if s.is_empty() || s.as_bytes()[0] != b'[' || s.as_bytes()[s.len() - 1] != b']' {
            Err(())
        } else {
            let s: &str = &s[1..(s.len() - 1)];
            if let Ok(v6) = IPv6Address::from_str(s) {
                Ok(SocketAddress::new(v6.to_ip(), port))
            } else {
                Err(())
            }
        }
    }
}
