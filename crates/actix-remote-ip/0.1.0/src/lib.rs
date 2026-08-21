//! #actix-remote-ip
//!
//! A tiny crate to determine the Real IP address of a client, taking account
//! of an eventual reverse proxy

use crate::ip_utils::{match_ip, parse_ip};
use actix_web::dev::Payload;
use actix_web::web::Data;
use actix_web::{Error, FromRequest, HttpRequest};
use futures_util::future::{ready, Ready};
use std::fmt::Display;
use std::net::IpAddr;

mod ip_utils;

/// Remote IP retrieval configuration
///
/// This configuration must be built and set as Actix web data
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct RemoteIPConfig {
    /// The IP address of the proxy. This address can ends with a star '*'
    pub proxy: Option<String>,
}

impl RemoteIPConfig {
    /// Initiate a new RemoteIPConfig configuration instance, that
    /// can be set as [actix_web::Data] structure
    pub fn with_proxy_ip<D: Display>(proxy: D) -> Self {
        Self {
            proxy: Some(proxy.to_string()),
        }
    }
}

/// Get the remote IP address
pub fn get_remote_ip(req: &HttpRequest) -> IpAddr {
    let proxy = req
        .app_data::<Data<RemoteIPConfig>>()
        .map(|c| c.proxy.as_ref())
        .unwrap_or_default();
    log::trace!("Proxy IP: {:?}", proxy);

    let mut ip = req.peer_addr().unwrap().ip();

    // We check if the request comes from a trusted reverse proxy
    if let Some(proxy) = proxy.as_ref() {
        if match_ip(proxy, &ip.to_string()) {
            if let Some(header) = req.headers().get("X-Forwarded-For") {
                let header = header.to_str().unwrap();

                let remote_ip = if let Some((upstream_ip, _)) = header.split_once(',') {
                    upstream_ip
                } else {
                    header
                };

                if let Some(upstream_ip) = parse_ip(remote_ip) {
                    ip = upstream_ip;
                }
            }
        }
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

    use crate::{get_remote_ip, RemoteIPConfig};
    use actix_web::test::TestRequest;
    use actix_web::web::Data;

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
    fn test_get_remote_ip_from_proxy() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1"))
            .app_data(Data::new(RemoteIPConfig::with_proxy_ip("192.168.1.1")))
            .to_http_request();

        assert_eq!(get_remote_ip(&req), "1.1.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_get_remote_ip_from_proxy_2() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1, 1.2.2.2"))
            .app_data(Data::new(RemoteIPConfig::with_proxy_ip("192.168.1.1")))
            .to_http_request();
        assert_eq!(get_remote_ip(&req), "1.1.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_get_remote_ip_from_proxy_ipv6() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "10::1, 1.2.2.2"))
            .app_data(Data::new(RemoteIPConfig::with_proxy_ip("192.168.1.1")))
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
    fn test_get_remote_ip_from_other_proxy() {
        let req = TestRequest::default()
            .peer_addr(SocketAddr::from_str("192.168.1.1:1000").unwrap())
            .insert_header(("X-Forwarded-For", "1.1.1.1, 1.2.2.2"))
            .app_data(Data::new(RemoteIPConfig::with_proxy_ip("192.168.1.2")))
            .to_http_request();

        assert_eq!(
            get_remote_ip(&req),
            "192.168.1.1".parse::<IpAddr>().unwrap()
        );
    }
}
