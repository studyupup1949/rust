use crate::{IPAddress, IPv4Address, IPv6Address};
use std::fmt::{Debug, Display, Formatter};

impl Debug for IPv4Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for IPv4Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_std(), f)
    }
}

impl Debug for IPv6Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for IPv6Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_std(), f)
    }
}

impl Debug for IPAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for IPAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V4(ip) => Display::fmt(ip, f),
            Self::V6(ip) => Display::fmt(ip, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn v4() {
        let test_cases: &[(IPv4Address, &str)] = &[
            (IPv4Address::UNSPECIFIED, "0.0.0.0"),
            (IPv4Address::LOCALHOST, "127.0.0.1"),
            (IPv4Address::BROADCAST, "255.255.255.255"),
        ];

        for (ip, expected) in test_cases {
            let result: String = ip.to_string();
            assert_eq!(result, *expected, "ip={:?}", ip);
        }
    }

    #[test]
    fn v6() {
        let test_cases: &[(IPv6Address, &str)] = &[(IPv6Address::UNSPECIFIED, "::"), (IPv6Address::LOCALHOST, "::1")];

        for (ip, expected) in test_cases {
            let result: String = ip.to_string();
            assert_eq!(result, *expected, "ip={:?}", ip);
        }
    }

    #[test]
    fn ip() {
        let test_cases: &[(IPAddress, &str)] = &[
            (IPv4Address::LOCALHOST.to_ip(), "127.0.0.1"),
            (IPv6Address::LOCALHOST.to_ip(), "::1"),
        ];

        for (ip, expected) in test_cases {
            let result: String = ip.to_string();
            assert_eq!(result, *expected, "ip={:?}", ip);
        }
    }
}
