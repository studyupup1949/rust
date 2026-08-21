//! Blocking DNS resolution via the platform's `getaddrinfo(3)`.
//!
//! This module is feature-gated behind `resolve`. It relies entirely on
//! [`std::net::ToSocketAddrs`] — there is no third-party DNS client and
//! no async runtime. Resolution honors `/etc/hosts`, `/etc/resolv.conf`,
//! mDNS, and any NSS plugins configured on the host.
//!
//! For non-A/AAAA record types or non-blocking resolution, use a
//! dedicated DNS crate such as `hickory-resolver`.

use std::io;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use crate::{Addr, Host};

impl Addr {
    /// Resolve the host component to one or more [`SocketAddr`]s.
    ///
    /// Uses the scheme's default port when no explicit port is set; if
    /// neither is available, port `0` is used.
    ///
    /// This call blocks and may touch the network. Wrap in
    /// `tokio::task::spawn_blocking` or equivalent from async contexts.
    pub fn resolve(&self) -> io::Result<Vec<SocketAddr>> {
        let port = self.effective_port().unwrap_or(0);
        match &self.host {
            Host::Ipv4(ip) => Ok(vec![SocketAddr::from((*ip, port))]),
            Host::Ipv6(ip) => Ok(vec![SocketAddr::from((*ip, port))]),
            Host::Domain(d) => (d.as_str(), port).to_socket_addrs().map(Iterator::collect),
        }
    }

    /// Like [`resolve`](Self::resolve) but returns only the IP addresses,
    /// dropping port information.
    pub fn resolve_ips(&self) -> io::Result<Vec<IpAddr>> {
        Ok(self.resolve()?.into_iter().map(|sa| sa.ip()).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::Addr;

    #[test]
    fn resolves_ipv4_literal() {
        let a = Addr::parse("http://127.0.0.1:8080/").unwrap();
        let addrs = a.resolve().unwrap();
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].port(), 8080);
    }

    #[test]
    fn resolves_ipv6_literal() {
        let a = Addr::parse("https://[::1]:9200/").unwrap();
        let addrs = a.resolve().unwrap();
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].port(), 9200);
    }
}
