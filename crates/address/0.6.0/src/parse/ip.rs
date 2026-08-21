use std::net::Ipv6Addr;
use std::str::FromStr;

use crate::{IPAddress, IPv4Address, IPv6Address};

impl FromStr for IPv4Address {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s: &str = s;
        let mut address: [u8; 4] = [0u8; 4];
        for i in 0..3 {
            match s.as_bytes().iter().position(|c| *c == b'.') {
                Some(dot) => {
                    address[i] = u8::from_str(&s[..dot]).map_err(|_| ())?;
                    s = &s[dot + 1..];
                }
                None => return Err(()),
            }
        }
        address[3] = u8::from_str(s).map_err(|_| ())?;
        Ok(Self::new(address))
    }
}

impl FromStr for IPv6Address {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ipv6Addr::from_str(s).map_err(|_| ()).map(IPv6Address::from)
    }
}

impl FromStr for IPAddress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v4) = IPv4Address::from_str(s) {
            Ok(v4.to_ip())
        } else if let Ok(v6) = IPv6Address::from_str(s) {
            Ok(v6.to_ip())
        } else {
            Err(())
        }
    }
}
