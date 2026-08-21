//! Whois lookups via the `whoizz` crate.
//!
//! Gated on the `whois` feature. Dispatches on the host component:
//! - [`Host::Domain`] → [`whoizz::lookup`] (TLD registry)
//! - [`Host::Ipv4`] / [`Host::Ipv6`] → [`whoizz::lookup_ip`] (RIR)

pub use whoizz::{WhoisError, WhoisResponse};

use std::net::IpAddr;

use crate::{Addr, Host};

impl Addr {
    /// Look up whois information for this address.
    ///
    /// Blocks on the network. Domain hosts hit the TLD registry;
    /// IP literals hit the Regional Internet Registry that owns the
    /// block.
    pub fn whois(&self) -> Result<WhoisResponse, WhoisError> {
        match &self.host {
            Host::Domain(d) => whoizz::lookup(d),
            Host::Ipv4(ip) => whoizz::lookup_ip(IpAddr::V4(*ip)),
            Host::Ipv6(ip) => whoizz::lookup_ip(IpAddr::V6(*ip)),
        }
    }
}
