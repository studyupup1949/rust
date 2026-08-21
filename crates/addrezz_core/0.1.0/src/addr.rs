use core::fmt;

use crate::{Host, ParseError, ParseOptions, Scheme, Userinfo, parser};

/// A parsed, normalized address.
///
/// Covers the full RFC 3986 URI shape: every supported input (URL, FQDN,
/// IP, IPv6, SSH/SCP-style) is normalized into this struct.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Addr {
    /// URI scheme (e.g. https, ssh)
    pub scheme: Scheme,
    /// Optional username and password
    pub userinfo: Option<Userinfo>,
    /// Domain name or IP address
    pub host: Host,
    /// Explicit port, if present
    pub port: Option<u16>,
    /// Path component of the URI
    pub path: String,
    /// Query string without leading `?`
    pub query: Option<String>,
    /// Fragment without leading `#`
    pub fragment: Option<String>,
}

impl Addr {
    /// Parse with default options.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parser::parse(input, &ParseOptions::default())
    }

    /// Parse with explicit options.
    pub fn parse_with(input: &str, opts: &ParseOptions) -> Result<Self, ParseError> {
        parser::parse(input, opts)
    }

    /// Return the port, falling back to the scheme's IANA default.
    pub fn effective_port(&self) -> Option<u16> {
        self.port.or_else(|| self.scheme.default_port())
    }

    /// True if the host component is a loopback or private-network address.
    pub fn is_local(&self) -> bool {
        self.host.is_local()
    }

    /// Serialize the value suitable for an HTTP `Host` request header
    /// (RFC 7230 §5.4). The port is elided when it matches the scheme
    /// default.
    pub fn host_header_string(&self) -> String {
        let host = self.host.to_string();
        match self.port {
            Some(p) if Some(p) != self.scheme.default_port() => format!("{host}:{p}"),
            _ => host,
        }
    }

    /// Iterate over percent-decoded path segments (the parts between
    /// `/` delimiters). Leading and trailing empty segments are skipped.
    pub fn path_segments(&self) -> impl Iterator<Item = std::borrow::Cow<'_, str>> {
        self.path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| percent_encoding::percent_decode_str(s).decode_utf8_lossy())
    }

    /// Iterate over query-string key-value pairs, decoded per
    /// `application/x-www-form-urlencoded`.
    ///
    /// Returns an empty iterator when no query string is present.
    pub fn query_pairs(&self) -> form_urlencoded::Parse<'_> {
        form_urlencoded::parse(self.query.as_deref().unwrap_or("").as_bytes())
    }

    /// Serialize only the scheme + authority (no path/query/fragment).
    pub fn base_url(&self) -> String {
        let mut s = String::with_capacity(32);
        s.push_str(self.scheme.as_str());
        s.push_str("://");
        if let Some(ui) = &self.userinfo {
            s.push_str(&ui.to_string());
            s.push('@');
        }
        s.push_str(&self.host.to_string());
        if let Some(p) = self.port {
            s.push(':');
            s.push_str(&p.to_string());
        }
        s
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.base_url())?;
        f.write_str(&self.path)?;
        if let Some(q) = &self.query {
            f.write_str("?")?;
            f.write_str(q)?;
        }
        if let Some(fr) = &self.fragment {
            f.write_str("#")?;
            f.write_str(fr)?;
        }
        Ok(())
    }
}

impl core::str::FromStr for Addr {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for Addr {
    type Error = ParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl TryFrom<String> for Addr {
    type Error = ParseError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_https() {
        let a = Addr::parse("https://github.com/").unwrap();
        assert_eq!(a.scheme, Scheme::Https);
        assert!(matches!(a.host, Host::Domain(ref d) if d == "github.com"));
    }

    #[test]
    fn scheme_less_defaults_to_https() {
        let a = Addr::parse("github.com").unwrap();
        #[cfg(not(feature = "default_http"))]
        assert_eq!(a.scheme, Scheme::Https);
    }

    #[test]
    fn host_and_port_without_scheme() {
        let a = Addr::parse("github.com:443").unwrap();
        #[cfg(not(feature = "default_http"))]
        assert_eq!(a.scheme, Scheme::Https);
        // url::Url normalizes default ports away; effective_port recovers them.
        assert_eq!(a.effective_port(), Some(443));
    }

    #[test]
    fn host_and_nondefault_port() {
        let a = Addr::parse("github.com:8443").unwrap();
        #[cfg(not(feature = "default_http"))]
        assert_eq!(a.scheme, Scheme::Https);
        assert_eq!(a.port, Some(8443));
    }

    #[test]
    fn scp_form_parses_as_ssh() {
        let a = Addr::parse("git@github.com:seq-rs/addrezz").unwrap();
        assert_eq!(a.scheme, Scheme::Ssh);
        assert_eq!(a.userinfo.as_ref().unwrap().username, "git");
        assert!(matches!(a.host, Host::Domain(ref d) if d == "github.com"));
        assert_eq!(a.path, "/seq-rs/addrezz");
    }

    #[test]
    fn ssh_full_form() {
        let a = Addr::parse("ssh://git@github.com/seq-rs/addrezz").unwrap();
        assert_eq!(a.scheme, Scheme::Ssh);
        assert_eq!(a.effective_port(), Some(22));
    }

    #[test]
    fn ipv4_literal() {
        let a = Addr::parse("http://127.0.0.1:8080/foo").unwrap();
        assert!(matches!(a.host, Host::Ipv4(_)));
        assert!(a.is_local());
    }

    #[test]
    fn ipv6_literal() {
        let a = Addr::parse("https://[::1]:9200").unwrap();
        assert!(matches!(a.host, Host::Ipv6(_)));
        assert!(a.is_local());
    }

    #[test]
    fn effective_port_falls_back_to_default() {
        let a = Addr::parse("https://example.com").unwrap();
        assert_eq!(a.port, None);
        assert_eq!(a.effective_port(), Some(443));
    }

    #[test]
    fn empty_input_errors() {
        assert!(matches!(Addr::parse("  "), Err(ParseError::Empty)));
    }

    #[test]
    fn host_header_elides_default_port() {
        let a = Addr::parse("https://example.com:443/foo").unwrap();
        assert_eq!(a.host_header_string(), "example.com");
    }

    #[test]
    fn host_header_keeps_nondefault_port() {
        let a = Addr::parse("https://example.com:8443/foo").unwrap();
        assert_eq!(a.host_header_string(), "example.com:8443");
    }

    #[test]
    fn path_segments_decodes() {
        let a = Addr::parse("https://example.com/foo/bar%20baz/qux").unwrap();
        let segs: Vec<_> = a.path_segments().collect();
        assert_eq!(segs, vec!["foo", "bar baz", "qux"]);
    }

    #[test]
    fn query_pairs_parses() {
        let a = Addr::parse("https://example.com/?key=val&a=b%20c").unwrap();
        let pairs: Vec<_> = a.query_pairs().collect();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("key".into(), "val".into()));
        assert_eq!(pairs[1], ("a".into(), "b c".into()));
    }

    #[test]
    fn query_pairs_empty_when_no_query() {
        let a = Addr::parse("https://example.com/").unwrap();
        assert_eq!(a.query_pairs().count(), 0);
    }

    #[test]
    #[cfg(feature = "schemars")]
    fn generates_schema() {
        let schema = schemars::schema_for!(Addr);
        let json = serde_json::to_string_pretty(&schema).unwrap();
            assert!(json.contains("scheme"));
    }
}
