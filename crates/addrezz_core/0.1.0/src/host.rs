use core::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

#[cfg(feature = "idna")]
use crate::HostError;

/// A host component: a registered domain name or an IP literal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum Host {
    /// A registered domain name
    Domain(String),
    /// An IPv4 address
    Ipv4(Ipv4Addr),
    /// An IPv6 address
    Ipv6(Ipv6Addr),
}

impl Host {
    /// True if the host is a loopback, link-local, RFC1918, CGNAT, or
    /// suffixed by `.local`/`.localhost`.
    pub fn is_local(&self) -> bool {
        match self {
            Self::Domain(d) => {
                let d = d.to_ascii_lowercase();
                d == "localhost" || d.ends_with(".localhost") || d.ends_with(".local")
            }
            Self::Ipv4(v4) => is_local_v4(*v4),
            Self::Ipv6(v6) => is_local_v6(*v6),
        }
    }

    /// True if this host is an IP literal.
    pub fn is_ip(&self) -> bool {
        matches!(self, Self::Ipv4(_) | Self::Ipv6(_))
    }
}

#[cfg(feature = "idna")]
impl Host {

    /// Convert host to human-readable string for display. Faulty conversions produce best-effort string,
    /// ignoring the thrown error.
    ///
    /// Wire form expect the [`std::fmt::Display`] impl's punycode form, this is purely for friendly
    /// representation.
    pub fn to_unicode(&self) -> String {
        match self {
            Self::Domain(d) => idna::domain_to_unicode(d).0,
            other => other.to_string(),
        }
    }

    pub fn try_to_unicode(&self) -> Result<String, HostError> {
        match self {
            Self::Domain(d) => {
                match idna::domain_to_unicode(d) {
                    (u, Ok(_)) => Ok(u),
                    (u, Err(_)) => Err(HostError::ConversionError(u.to_string())),
                }
            }
            other => Ok(other.to_string()),
        }
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(d) => f.write_str(d),
            Self::Ipv4(ip) => write!(f, "{ip}"),
            Self::Ipv6(ip) => write!(f, "[{ip}]"),
        }
    }
}

fn is_local_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    match o[0] {
        10 => true,                                // 10.0.0.0/8    RFC1918
        127 => true,                               // 127.0.0.0/8   loopback
        172 if (16..=31).contains(&o[1]) => true,  // 172.16.0.0/12 RFC1918
        192 if o[1] == 168 => true,                // 192.168.0.0/16 RFC1918
        100 if (64..=127).contains(&o[1]) => true, // 100.64.0.0/10 CGNAT
        169 if o[1] == 254 => true,                // 169.254.0.0/16 link-local
        _ => false,
    }
}

fn is_local_v6(ip: Ipv6Addr) -> bool {
    if ip.is_loopback() {
        return true;
    }
    let seg0 = ip.segments()[0];
    seg0 & 0xfe00 == 0xfc00 || seg0 & 0xffc0 == 0xfe80
}
