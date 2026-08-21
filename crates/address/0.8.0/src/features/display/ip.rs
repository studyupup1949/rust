use std::fmt::{Display, Formatter};

use crate::{IPAddress, IPv4Address, IPv6Address};

impl Display for IPv4Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (a, b, c, d) = self.bytes();
        write!(f, "{}.{}.{}.{}", a, b, c, d)
    }
}

impl Display for IPv6Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_std())
    }
}

impl Display for IPAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V4(v4) => write!(f, "{}", v4),
            Self::V6(v6) => write!(f, "{}", v6),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{IPAddress, IPv4Address, IPv6Address};

    #[test]
    fn v4() {
        let ip: IPv4Address = IPv4Address::LOCALHOST;
        assert_eq!(ip.to_string(), "127.0.0.1");
    }

    #[test]
    fn v6() {
        let ip: IPv6Address = IPv6Address::LOCALHOST;
        assert_eq!(ip.to_string(), "::1");
    }

    #[test]
    fn ip() {
        let ip: IPAddress = IPv4Address::LOCALHOST.to_ip();
        assert_eq!(ip.to_string(), "127.0.0.1");

        let ip: IPAddress = IPv6Address::LOCALHOST.to_ip();
        assert_eq!(ip.to_string(), "::1");
    }
}
