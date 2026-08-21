use core::fmt;

use crate::{Addr, Host, Scheme};

/// An RFC 6454 origin — the (scheme, host, port) triple with no
/// userinfo, path, query, or fragment.
///
/// Two origins compare equal when all three components match.
/// `effective_port` fills in the scheme default when no explicit port
/// is set, which is useful for CORS / CSP / cookie comparisons where
/// the default port is implied.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Origin {
    /// URI scheme
    pub scheme: Scheme,
    /// Domain name or IP address
    pub host: Host,
    /// Explicit port, if present
    pub port: Option<u16>,
}

impl Origin {
    /// Return the port, falling back to the scheme's IANA default.
    pub fn effective_port(&self) -> Option<u16> {
        self.port.or_else(|| self.scheme.default_port())
    }

    /// True if both origins share the same scheme, host, and effective
    /// port — the standard same-origin check.
    pub fn same_origin(&self, other: &Origin) -> bool {
        self.scheme == other.scheme
            && self.host == other.host
            && self.effective_port() == other.effective_port()
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}", self.scheme, self.host)?;
        if let Some(p) = self.port {
            write!(f, ":{p}")?;
        }
        Ok(())
    }
}

impl Addr {
    /// Extract the RFC 6454 origin — scheme + host + port, stripping
    /// userinfo, path, query, and fragment.
    pub fn origin(&self) -> Origin {
        Origin {
            scheme: self.scheme.clone(),
            host: self.host.clone(),
            port: self.port,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Addr;

    #[test]
    fn origin_strips_path_and_userinfo() {
        let a = Addr::parse("https://user:pass@example.com:8443/foo?bar=1#baz").unwrap();
        let o = a.origin();
        assert_eq!(o.to_string(), "https://example.com:8443");
    }

    #[test]
    fn same_origin_ignores_explicit_default_port() {
        let a = Addr::parse("https://example.com/").unwrap();
        let b = Addr::parse("https://example.com:443/other").unwrap();
        assert!(a.origin().same_origin(&b.origin()));
    }

    #[test]
    fn different_scheme_is_cross_origin() {
        let a = Addr::parse("http://example.com/").unwrap();
        let b = Addr::parse("https://example.com/").unwrap();
        assert!(!a.origin().same_origin(&b.origin()));
    }
}
