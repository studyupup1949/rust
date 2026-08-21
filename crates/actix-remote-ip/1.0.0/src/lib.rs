//! #actix-remote-ip
//!
//! A tiny crate to determine the Real IP address of a client, taking account
//! of an eventual reverse proxy

use crate::legacy_ip_utils::legacy_match_ip;
use actix_web::dev::Payload;
use actix_web::web::Data;
use actix_web::{Error, FromRequest, HttpRequest};
use futures_util::future::{Ready, ready};
use ipnet::AddrParseError;
use std::fmt::Display;
use std::net::{IpAddr, Ipv6Addr};
use std::str::FromStr;

/// Legacy IP support
pub(crate) mod legacy_ip_utils;

/// Remote IP retrieval configuration
///
/// This configuration must be built and set as Actix web data
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum RemoteIPConfig {
    /// Legacy IP format
    Legacy(String),
    /// Modern IP format
    Modern(Vec<ipnet::IpNet>),
    #[default]
    None,
}

impl RemoteIPConfig {
    /// Initiate a new RemoteIPConfig configuration instance, that
    /// can be set as [actix_web::Data] structure
    pub fn parse_opt<D: Display>(proxy: Option<D>) -> Self {
        match proxy {
            None => Self::None,
            Some(p) => Self::parse(p),
        }
    }

    /// Initiate a new RemoteIPConfig configuration instance, that
    /// can be set as [actix_web::Data] structure
    pub fn parse<D: Display>(proxy: D) -> Self {
        Self::try_parse(proxy).expect("failed to parse given proxy ip address!")
    }

    /// Initiate a new RemoteIPConfig configuration instance, that
    /// can be set as [actix_web::Data] structure
    pub fn try_parse<D: Display>(proxy: D) -> Result<Self, AddrParseError> {
        let proxy_ip = proxy.to_string();

        // If nothing was specified...
        if proxy_ip.trim().is_empty() {
            return Ok(Self::None);
        }

        // Handle legacy format
        if (proxy_ip.contains("*") || !proxy_ip.contains("/")) && !proxy_ip.contains(",") {
            return Ok(Self::Legacy(proxy_ip));
        }

        let mut ips = vec![];

        for ip in proxy_ip.split(",") {
            let ip = ip.trim();
            if ip.is_empty() {
                continue;
            }

            ips.push(ipnet::IpNet::from_str(ip)?);
        }

        Ok(Self::Modern(ips))
    }

    /// Check out whether an IP address is a trusted IP address or not
    pub fn is_trusted_proxy_ip(&self, ip: IpAddr) -> bool {
        match self {
            RemoteIPConfig::Legacy(proxy_ip) => legacy_match_ip(proxy_ip, &ip.to_string()),

            RemoteIPConfig::Modern(networks) => networks.iter().any(|n| n.contains(&ip)),

            // No proxy
            RemoteIPConfig::None => false,
        }
    }
}

/// Get the remote IP address
pub fn get_remote_ip(req: &HttpRequest) -> IpAddr {
    let config = req.app_data::<Data<RemoteIPConfig>>().map(|c| c.as_ref());

    log::trace!("Proxy IP config: {:?}", config);

    let mut peer_ip = req.peer_addr().unwrap().ip();

    // We check if the request comes from a trusted reverse proxy
    if let Some(config) = config.as_ref()
        && config.is_trusted_proxy_ip(peer_ip)
        && let Some(header) = req.headers().get("X-Forwarded-For")
    {
        let header = header.to_str().unwrap();

        let remote_ip = if let Some((upstream_ip, _)) = header.split_once(',') {
            upstream_ip
        } else {
            header
        };

        match IpAddr::from_str(remote_ip) {
            Ok(ip) => peer_ip = ip,
            Err(e) => {
                log::warn!("Failed to parse an IP address: {e}");
            }
        }
    }

    generalize_ip(peer_ip)
}

/// Generalize IPv6 to consider IP inside a network as a single IP (for bruteforce protection)
fn generalize_ip(mut ip: IpAddr) -> IpAddr {
    // In case of IPv6 address, we skip the 8 last octets
    if let IpAddr::V6(ipv6) = &mut ip {
        let mut octets = ipv6.octets();
        for o in octets.iter_mut().skip(8) {
            *o = 0;
        }
        ip = IpAddr::V6(Ipv6Addr::from(octets));
    }

    ip
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RemoteIP(pub IpAddr);

impl From<RemoteIP> for IpAddr {
    fn from(i: RemoteIP) -> Self {
        i.0
    }
}

impl FromRequest for RemoteIP {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(Ok(RemoteIP(get_remote_ip(req))))
    }
}

#[cfg(test)]
mod test {
    use std::net::{IpAddr, SocketAddr};
    use std::str::FromStr;

    use crate::{RemoteIPConfig, get_remote_ip};
    use actix_web::test::TestRequest;
    use actix_web::web::Data;
    use ipnet::{IpNet, Ipv4Net, Ipv6Net};

    #[test]
    fn test_parse_config() {
        assert_eq!(
            RemoteIPConfig::parse_opt::<String>(None),
            RemoteIPConfig::None
        );
        assert_eq!(
            RemoteIPConfig::parse_opt(Some("192.168.1.5")),
            RemoteIPConfig::Legacy("192.168.1.5".to_string())
        );
        assert_eq!(
            RemoteIPConfig::parse("192.168.1.5"),
            RemoteIPConfig::Legacy("192.168.1.5".to_string())
        );
        assert_eq!(
            RemoteIPConfig::try_parse("192.168.1.5"),
            Ok(RemoteIPConfig::Legacy("192.168.1.5".to_string()))
        );

        assert_eq!(
            RemoteIPConfig::parse("10.0.0.0/8"),
            RemoteIPConfig::Modern(vec![IpNet::V4(Ipv4Net::from_str("10.0.0.0/8").unwrap())])
        );
        assert_eq!(
            RemoteIPConfig::parse("10.0.0.0/8,192.168.0.0/16"),
            RemoteIPConfig::Modern(vec![
                IpNet::V4(Ipv4Net::from_str("10.0.0.0/8").unwrap()),
                IpNet::V4(Ipv4Net::from_str("192.168.0.0/16").unwrap())
            ])
        );
        assert_eq!(
            RemoteIPConfig::parse("10.0.0.0/8,192.168.0.0/16,2001:d445:b62e:dd2a::/64"),
            RemoteIPConfig::Modern(vec![
                IpNet::V4(Ipv4Net::from_str("10.0.0.0/8").unwrap()),
                IpNet::V4(Ipv4Net::from_str("192.168.0.0/16").unwrap()),
                IpNet::V6(Ipv6Net::from_str("2001:d445:b62e:dd2a::/64").unwrap())
            ])
        );
        assert_eq!(RemoteIPConfig::parse(""), RemoteIPConfig::None);

        assert!(RemoteIPConfig::try_parse("10.0.0.0/a,192.168.0.0/16").is_err());
        assert!(RemoteIPConfig::try_parse("10.0.0.0/a,192:.168.0.0/16").is_err());
        assert!(RemoteIPConfig::try_parse("10.0.0.0/a,192:*.168.0.0/16").is_err());
    }

    #[test]
    fn test_trusted_ip() {
        let none = RemoteIPConfig::None;
        assert!(!none.is_trusted_proxy_ip(IpAddr::from_str("0.0.0.0").unwrap()));
        assert!(!none.is_trusted_proxy_ip(IpAddr::from_str("10.20.30.40").unwrap()));
        assert!(!none.is_trusted_proxy_ip(IpAddr::from_str("192.168.1.0").unwrap()));
        assert!(!none.is_trusted_proxy_ip(IpAddr::from_str("192.169.1.0").unwrap()));
        assert!(
            !none.is_trusted_proxy_ip(IpAddr::from_str("2001:d445:b62e:dd2a:2:2:2:0").unwrap())
        );

        let legacy = RemoteIPConfig::Legacy("10.*".to_string());
        assert!(legacy.is_trusted_proxy_ip(IpAddr::from_str("10.20.30.40").unwrap()));
        assert!(!legacy.is_trusted_proxy_ip(IpAddr::from_str("192.168.1.0").unwrap()));
        assert!(!legacy.is_trusted_proxy_ip(IpAddr::from_str("192.169.1.0").unwrap()));
        assert!(
            !legacy.is_trusted_proxy_ip(IpAddr::from_str("2001:d445:b62e:dd2a:2:2:2:0").unwrap())
        );

        let legacyv6 = RemoteIPConfig::Legacy("2001:d445:b62e:dd2a*".to_string());
        assert!(!legacyv6.is_trusted_proxy_ip(IpAddr::from_str("10.20.30.40").unwrap()));
        assert!(!legacyv6.is_trusted_proxy_ip(IpAddr::from_str("192.168.1.0").unwrap()));
        assert!(!legacyv6.is_trusted_proxy_ip(IpAddr::from_str("192.169.1.0").unwrap()));
        assert!(
            legacyv6.is_trusted_proxy_ip(IpAddr::from_str("2001:d445:b62e:dd2a:2:2:2:0").unwrap())
        );
        assert!(
            !legacyv6.is_trusted_proxy_ip(IpAddr::from_str("2001:a445:b62e:dd2a:2:2:2:0").unwrap())
        );

        let modern = RemoteIPConfig::Modern(vec![
            IpNet::V4(Ipv4Net::from_str("10.0.0.0/8").unwrap()),
            IpNet::V4(Ipv4Net::from_str("192.168.0.0/16").unwrap()),
            IpNet::V6(Ipv6Net::from_str("2001:d445:b62e:dd2a::/64").unwrap()),
        ]);
        assert!(modern.is_trusted_proxy_ip(IpAddr::from_str("10.20.30.40").unwrap()));
        assert!(modern.is_trusted_proxy_ip(IpAddr::from_str("192.168.1.0").unwrap()));
        assert!(!modern.is_trusted_proxy_ip(IpAddr::from_str("192.169.1.0").unwrap()));
        assert!(
            modern.is_trusted_proxy_ip(IpAddr::from_str("2001:d445:b62e:dd2a:2:2:2:0").unwrap())
        );
    }

    #[test]
    fn test_get_remote_ip() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .to_http_request();
        assert_eq!(
            get_remote_ip(&req),
            "192.168.1.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_get_remote_ip_v6() {
        let req = TestRequest::default()
            .peer_addr(
                SocketAddr::from_str("[2001:c0c3:f0d2:562d:193d:a5f6:1668:3cba]:1000").unwrap(),
            )
            .to_http_request();
        assert_eq!(
            get_remote_ip(&req),
            "2001:c0c3:f0d2:562d:0:0:0:0".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_get_remote_ip_from_proxy_legacy() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1"))
            .app_data(Data::new(RemoteIPConfig::parse("192.168.1.1")))
            .to_http_request();

        assert_eq!(get_remote_ip(&req), "1.1.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_get_remote_ip_from_proxy_legacy_2() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1, 1.2.2.2"))
            .app_data(Data::new(RemoteIPConfig::parse("192.168.1.1")))
            .to_http_request();
        assert_eq!(get_remote_ip(&req), "1.1.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_get_remote_ip_from_proxy_modern() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1"))
            .app_data(Data::new(RemoteIPConfig::parse("192.168.1.0/24")))
            .to_http_request();

        assert_eq!(get_remote_ip(&req), "1.1.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_get_remote_ip_from_proxy_modern_multiple() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("10.10.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1"))
            .app_data(Data::new(RemoteIPConfig::parse(
                "192.168.1.0/24,10.10.0.0/16",
            )))
            .to_http_request();

        assert_eq!(get_remote_ip(&req), "1.1.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_get_remote_ip_from_proxy_modern_untrusted() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("10.10.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1"))
            .app_data(Data::new(RemoteIPConfig::parse("192.168.1.0/24")))
            .to_http_request();

        assert_eq!(get_remote_ip(&req), "10.10.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_get_remote_ip_from_proxy_modern_untrusted_ipv6() {
        let req = TestRequest::default()
            .peer_addr(
                SocketAddr::from_str("[2001:d445:b62e:dd2a:1ab8:3a21:c200:69ce]:1000").unwrap(),
            )
            .insert_header(("X-Forwarded-For", "1.1.1.1"))
            .app_data(Data::new(RemoteIPConfig::parse("192.168.1.0/24")))
            .to_http_request();

        assert_eq!(
            get_remote_ip(&req),
            "2001:d445:b62e:dd2a::".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_get_remote_ip_from_proxy_modern_trusted_ipv6() {
        let req = TestRequest::default()
            .peer_addr(
                SocketAddr::from_str("[2001:d445:b62e:dd2a:1ab8:3a21:c200:69ce]:1000").unwrap(),
            )
            .insert_header(("X-Forwarded-For", "2001:129c:4dee:1f22:ddcb:d2ae:cdcb:dd22"))
            .app_data(Data::new(RemoteIPConfig::parse(
                "192.168.1.0/24,2001:d445:b62e:dd2a:1ab8:3a21:c200:69ce/128",
            )))
            .to_http_request();

        assert_eq!(
            get_remote_ip(&req),
            "2001:129c:4dee:1f22::".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_get_remote_ip_from_proxy_ipv6_legacy() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "10::1, 1.2.2.2"))
            .app_data(Data::new(RemoteIPConfig::parse("192.168.1.1")))
            .to_http_request();

        assert_eq!(get_remote_ip(&req), "10::".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_get_remote_ip_from_no_proxy() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1, 1.2.2.2"))
            .to_http_request();

        assert_eq!(
            get_remote_ip(&req),
            "192.168.1.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_get_remote_ip_from_other_proxy_legacy() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1, 1.2.2.2"))
            .app_data(Data::new(RemoteIPConfig::parse("192.168.1.2")))
            .to_http_request();

        assert_eq!(
            get_remote_ip(&req),
            "192.168.1.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_get_remote_ip_from_other_proxy_modern() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1, 1.2.2.2"))
            .app_data(Data::new(RemoteIPConfig::parse("192.168.1.2/32")))
            .to_http_request();

        assert_eq!(
            get_remote_ip(&req),
            "192.168.1.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_get_remote_ip_config_without_proxy_legacy() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .app_data(Data::new(RemoteIPConfig::parse("10.0.0.*")))
            .to_http_request();

        assert_eq!(
            get_remote_ip(&req),
            "192.168.1.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_get_remote_ip_config_without_proxy_modern() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .app_data(Data::new(RemoteIPConfig::parse("10.0.0.1/32")))
            .to_http_request();

        assert_eq!(
            get_remote_ip(&req),
            "192.168.1.1".parse::<IpAddr>().unwrap()
        );
    }
}
