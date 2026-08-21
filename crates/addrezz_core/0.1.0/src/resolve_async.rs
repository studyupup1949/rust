//! Async DNS resolution via `hickory-resolver`.
//!
//! Feature-gated behind `resolve_async`. Unlike the sync [`Addr::resolve`]
//! path (which uses libc `getaddrinfo`), this uses a pure-Rust DNS
//! client. It is async and supports non-A/AAAA record types if you
//! drop down to the underlying resolver directly.
//!
//! The system resolver (`/etc/resolv.conf`, or the registry on Windows)
//! is always preferred. The [`ResolverConfig`] only comes into play as a
//! fallback when that system configuration cannot be read. Presets for the
//! common public resolvers are re-exported: [`CLOUDFLARE`], [`GOOGLE`],
//! [`QUAD9`].

use std::io;
use std::net::SocketAddr;

use hickory_resolver::net::runtime::TokioRuntimeProvider;
use hickory_resolver::{Resolver, TokioResolver};

pub use hickory_resolver::config::{CLOUDFLARE, GOOGLE, QUAD9, ResolverConfig};

use crate::{Addr, Host};

impl Addr {
    /// Asynchronously resolve the host component to socket addresses.
    ///
    /// Uses the system resolver configuration when possible, falling back to
    /// Cloudflare (1.1.1.1) if it cannot be read. The port is filled from the
    /// address's effective port, or 0 if none is known.
    ///
    /// To pick a different fallback resolver, use [`Addr::resolve_async_with`].
    pub async fn resolve_async(&self) -> io::Result<Vec<SocketAddr>> {
        self.resolve_async_with(ResolverConfig::udp_and_tcp(&CLOUDFLARE))
            .await
    }

    /// Like [`Addr::resolve_async`], but with a caller-chosen fallback config.
    ///
    /// `fallback` is used only when the system resolver configuration cannot
    /// be read; the normal path still prefers the system resolver. Build it
    /// from a preset, e.g. `ResolverConfig::udp_and_tcp(&GOOGLE)`.
    pub async fn resolve_async_with(
        &self,
        fallback: ResolverConfig,
    ) -> io::Result<Vec<SocketAddr>> {
        let port = self.effective_port().unwrap_or(0);
        match &self.host {
            Host::Ipv4(ip) => Ok(vec![SocketAddr::from((*ip, port))]),
            Host::Ipv6(ip) => Ok(vec![SocketAddr::from((*ip, port))]),
            Host::Domain(d) => {
                let resolver = build_resolver(fallback)?;
                let lookup = resolver
                    .lookup_ip(d.as_str())
                    .await
                    .map_err(io::Error::other)?;
                Ok(lookup.iter().map(|ip| SocketAddr::new(ip, port)).collect())
            }
        }
    }
}

fn build_resolver(fallback: ResolverConfig) -> io::Result<TokioResolver> {
    // Prefer the host's system config; fall back to the given config if it
    // cannot be read.
    let builder = match TokioResolver::builder_tokio() {
        Ok(b) => b,
        Err(_) => Resolver::builder_with_config(fallback, TokioRuntimeProvider::default()),
    };
    builder.build().map_err(io::Error::other)
}
