use std::net::{IpAddr, Ipv6Addr};
use std::str::FromStr;

/// Parse an IP address
pub fn parse_ip(ip: &str) -> Option<IpAddr> {
    let mut ip = match IpAddr::from_str(ip) {
        Ok(ip) => ip,
        Err(e) => {
            log::warn!("Failed to parse an IP address: {}", e);
            return None;
        }
    };

    // In case of IPv6 address, we skip the 8 last octets
    if let IpAddr::V6(ipv6) = &mut ip {
        let mut octets = ipv6.octets();
        for o in octets.iter_mut().skip(8) {
            *o = 0;
        }
        ip = IpAddr::V6(Ipv6Addr::from(octets));
    }

    Some(ip)
}

/// Check if two ips matches
pub fn match_ip(pattern: &str, ip: &str) -> bool {
    if pattern.eq(ip) {
        return true;
    }

    if pattern.ends_with('*') && ip.starts_with(&pattern.replace('*', "")) {
        return true;
    }

    false
}

#[cfg(test)]
mod test {
    use crate::ip_utils::parse_ip;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn parse_bad_ip() {
        let ip = parse_ip("badbad");
        assert_eq!(None, ip);
    }

    #[test]
    fn parse_ip_v4_address() {
        let ip = parse_ip("192.168.1.1").unwrap();
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    }

    #[test]
    fn parse_ip_v6_address() {
        let ip = parse_ip("2a00:1450:4007:813::200e").unwrap();
        assert_eq!(
            ip,
            IpAddr::V6(Ipv6Addr::new(0x2a00, 0x1450, 0x4007, 0x813, 0, 0, 0, 0))
        );
    }

    #[test]
    fn parse_ip_v6_address_2() {
        let ip = parse_ip("::1").unwrap();
        assert_eq!(ip, IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)));
    }

    #[test]
    fn parse_ip_v6_address_3() {
        let ip = parse_ip("a::1").unwrap();
        assert_eq!(ip, IpAddr::V6(Ipv6Addr::new(0xa, 0, 0, 0, 0, 0, 0, 0)));
    }
}
