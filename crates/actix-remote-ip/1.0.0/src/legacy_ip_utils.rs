/// Check if two ips matches
pub(crate) fn legacy_match_ip(pattern: &str, ip: &str) -> bool {
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
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    /// Parse an IP address
    fn parse_ip(ip: &str) -> Option<IpAddr> {
        match IpAddr::from_str(ip) {
            Ok(ip) => Some(ip),
            Err(e) => {
                log::warn!("Failed to parse an IP address: {e}");
                None
            }
        }
    }

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
            IpAddr::V6(Ipv6Addr::new(
                0x2a00, 0x1450, 0x4007, 0x813, 0, 0, 0, 0x200e
            ))
        );
    }

    #[test]
    fn parse_ip_v6_address_2() {
        let ip = parse_ip("::1").unwrap();
        assert_eq!(ip, IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)));
    }

    #[test]
    fn parse_ip_v6_address_3() {
        let ip = parse_ip("a::1").unwrap();
        assert_eq!(ip, IpAddr::V6(Ipv6Addr::new(0xa, 0, 0, 0, 0, 0, 0, 1)));
    }
}
