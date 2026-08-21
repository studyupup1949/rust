//! CIDR / subnet membership via the `ipnet` crate.
//!
//! Gated behind the `ipnet` feature. Complements [`Host::is_local`] by
//! letting callers test against arbitrary user-defined network ranges.

use std::net::IpAddr;

use ipnet::IpNet;

use crate::{Addr, Host};

impl Host {
    /// True if this host is an IP literal that falls within the given
    /// CIDR range. Domain hosts always return `false` — resolve them
    /// first if you need IP-level checks.
    pub fn is_in_cidr(&self, cidr: &IpNet) -> bool {
        match self {
            Self::Ipv4(ip) => cidr.contains(&IpAddr::V4(*ip)),
            Self::Ipv6(ip) => cidr.contains(&IpAddr::V6(*ip)),
            Self::Domain(_) => false,
        }
    }
}

impl Addr {
    /// True if the address's host is an IP within the given CIDR.
    pub fn is_in_cidr(&self, cidr: &IpNet) -> bool {
        self.host.is_in_cidr(cidr)
    }
}

#[cfg(test)]
mod tests {
    use crate::Addr;

    #[test]
    fn ipv4_in_cidr() {
        let a = Addr::parse("http://10.0.0.42:8080/").unwrap();
        let cidr: ipnet::IpNet = "10.0.0.0/8".parse().unwrap();
        assert!(a.is_in_cidr(&cidr));
    }

    #[test]
    fn ipv4_not_in_cidr() {
        let a = Addr::parse("http://192.168.1.1/").unwrap();
        let cidr: ipnet::IpNet = "10.0.0.0/8".parse().unwrap();
        assert!(!a.is_in_cidr(&cidr));
    }

    #[test]
    fn domain_is_never_in_cidr() {
        let a = Addr::parse("https://example.com/").unwrap();
        let cidr: ipnet::IpNet = "0.0.0.0/0".parse().unwrap();
        assert!(!a.is_in_cidr(&cidr));
    }

    #[test]
    fn ipv6_in_cidr() {
        let a = Addr::parse("https://[fd12::1]:443/").unwrap();
        let cidr: ipnet::IpNet = "fd00::/8".parse().unwrap();
        assert!(a.is_in_cidr(&cidr));
    }
}
