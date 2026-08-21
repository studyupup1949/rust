use crate::{Addr, ParseError};

impl TryFrom<&Addr> for http::Uri {
    type Error = ParseError;
    fn try_from(a: &Addr) -> Result<Self, Self::Error> {
        a.to_string()
            .parse::<http::Uri>()
            .map_err(|e| ParseError::Invalid(e.to_string()))
    }
}

impl TryFrom<Addr> for http::Uri {
    type Error = ParseError;
    fn try_from(a: Addr) -> Result<Self, Self::Error> {
        (&a).try_into()
    }
}

impl TryFrom<&http::Uri> for Addr {
    type Error = ParseError;
    fn try_from(u: &http::Uri) -> Result<Self, Self::Error> {
        Addr::parse(&u.to_string())
    }
}

impl TryFrom<http::Uri> for Addr {
    type Error = ParseError;
    fn try_from(u: http::Uri) -> Result<Self, Self::Error> {
        (&u).try_into()
    }
}

/// Convert an `Addr` to an `http::uri::Authority` — the `[user@]host[:port]`
/// fragment of the URI.
impl TryFrom<&Addr> for http::uri::Authority {
    type Error = ParseError;
    fn try_from(a: &Addr) -> Result<Self, Self::Error> {
        let mut s = String::with_capacity(32);
        if let Some(ui) = &a.userinfo {
            s.push_str(&ui.to_string());
            s.push('@');
        }
        s.push_str(&a.host.to_string());
        if let Some(p) = a.port {
            s.push(':');
            s.push_str(&p.to_string());
        }
        s.parse::<http::uri::Authority>()
            .map_err(|e| ParseError::Invalid(e.to_string()))
    }
}

/// Produce an `http::HeaderValue` suitable for use as the value of the
/// HTTP `Host` request header. The port is elided when it matches the
/// scheme default.
impl TryFrom<&Addr> for http::HeaderValue {
    type Error = ParseError;
    fn try_from(a: &Addr) -> Result<Self, Self::Error> {
        http::HeaderValue::from_str(&a.host_header_string())
            .map_err(|e| ParseError::Invalid(e.to_string()))
    }
}

impl TryFrom<Addr> for http::HeaderValue {
    type Error = ParseError;
    fn try_from(a: Addr) -> Result<Self, Self::Error> {
        (&a).try_into()
    }
}
