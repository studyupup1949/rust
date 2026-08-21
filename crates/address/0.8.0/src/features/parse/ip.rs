use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use crate::{IPAddress, IPv4Address, IPv6Address};

impl FromStr for IPv4Address {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(IPv4Address::from(Ipv4Addr::from_str(s).map_err(|_| ())?))
    }
}

impl FromStr for IPv6Address {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(IPv6Address::from(Ipv6Addr::from_str(s).map_err(|_| ())?))
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn v4() {
        let test_cases: &[(&str, Result<IPv4Address, ()>)] = &[
            ("", Err(())),
            ("127.0.0", Err(())),
            ("0.0.1", Err(())),
            ("127.0.0.0.1", Err(())),
            ("x.0.0.1", Err(())),
            ("127.0.0.x", Err(())),
            ("::1", Err(())),
            ("127.0.0.1", Ok(IPv4Address::LOCALHOST)),
        ];
        for (s, expected) in test_cases {
            let result = IPv4Address::from_str(*s);
            assert_eq!(result, *expected, "{}", *s);
        }
    }

    #[test]
    fn v6() {
        let test_cases: &[(&str, Result<IPv6Address, ()>)] = &[
            ("", Err(())),
            ("0123:4567:89AB:CDEF:0123:4567:89AB:CDEF:0123", Err(())),
            ("0123:4567:89AB:CDEF:0123:4567:89AB", Err(())),
            ("0123:4567:89AB:CDEF:0123:4567:89AB:x", Err(())),
            ("x:4567:89AB:CDEF:0123:4567:89AB:CDEF", Err(())),
            ("127.0.0.1", Err(())),
            ("[::1]", Err(())),
            (
                "0123:4567:89AB:CDEF:0123:4567:89AB:CDEF",
                Ok(IPv6Address::from([
                    0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
                ])),
            ),
            ("::1", Ok(IPv6Address::LOCALHOST)),
        ];
        for (s, expected) in test_cases {
            let result = IPv6Address::from_str(*s);
            assert_eq!(result, *expected, "{}", s);
        }
    }

    #[test]
    fn ip() {
        let test_cases: &[(&str, Result<IPAddress, ()>)] = &[
            ("", Err(())),
            ("127.0.0", Err(())),
            ("0.0.1", Err(())),
            ("127.0.0.0.1", Err(())),
            ("x.0.0.1", Err(())),
            ("127.0.0.x", Err(())),
            ("0123:4567:89AB:CDEF:0123:4567:89AB:CDEF:0123", Err(())),
            ("0123:4567:89AB:CDEF:0123:4567:89AB", Err(())),
            ("0123:4567:89AB:CDEF:0123:4567:89AB:x", Err(())),
            ("x:4567:89AB:CDEF:0123:4567:89AB:CDEF", Err(())),
            ("[::1]", Err(())),
            ("127.0.0.1", Ok(IPv4Address::LOCALHOST.to_ip())),
            (
                "0123:4567:89AB:CDEF:0123:4567:89AB:CDEF",
                Ok(IPv6Address::from([
                    0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
                ])
                .to_ip()),
            ),
            ("::1", Ok(IPv6Address::LOCALHOST.to_ip())),
        ];
        for (s, expected) in test_cases {
            let result = IPAddress::from_str(*s);
            assert_eq!(result, *expected, "{}", s);
        }
    }
}
