use crate::{Addr, Host, ParseError, ParseOptions, Scheme, Userinfo};

pub(crate) fn parse(input: &str, opts: &ParseOptions) -> Result<Addr, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::Empty);
    }

    let has_scheme_marker = input.contains("://");

    // Attempt 1: parse as-is.
    if let Ok(url) = url::Url::parse(input) {
        if is_plausible_url(&url) {
            return from_url(url, opts);
        }
    }

    // If the input already looks like a proper URL (has "://"), don't
    // try scheme-rewriting heuristics — that would double the scheme
    // and produce nonsense. Report the failure instead.
    if has_scheme_marker {
        return Err(ParseError::Invalid(format!(
            "could not parse URL: {input:?}"
        )));
    }

    // Attempt 2: SCP-like `[user@]host:path` with non-numeric path.
    if opts.allow_scp_like {
        if let Some(rewritten) = scp_to_ssh(input) {
            if let Ok(url) = url::Url::parse(&rewritten) {
                return from_url(url, opts);
            }
        }
    }

    // Attempt 3: scheme-less — prepend the default scheme.
    let prepended = format!("{}://{}", opts.default_scheme.as_str(), input);
    let url = url::Url::parse(&prepended).map_err(|e| ParseError::Invalid(e.to_string()))?;
    from_url(url, opts)
}

fn is_plausible_url(url: &url::Url) -> bool {
    // `gitlab.com:443` parses as scheme=gitlab.com, path=443 — bogus.
    // Registered scheme syntax permits `.` but no real scheme uses it.
    !url.scheme().contains('.') && !url.scheme().is_empty()
}

/// Convert `git@host.com:path/to/repo` to `ssh://git@host.com/path/to/repo`.
/// Returns None if the input doesn't match the SCP shape.
fn scp_to_ssh(s: &str) -> Option<String> {
    if s.contains("://") {
        return None;
    }
    let at = s.find('@')?;
    let (user, rest) = (&s[..at], &s[at + 1..]);
    if user.is_empty() {
        return None;
    }
    let colon = rest.find(':')?;
    let (host, path) = (&rest[..colon], &rest[colon + 1..]);
    if host.is_empty() {
        return None;
    }
    // If the "path" is an integer port, it's not SCP form.
    if !path.is_empty() && path.chars().all(|c| c.is_ascii_digit()) && path.parse::<u16>().is_ok() {
        return None;
    }
    Some(format!("ssh://{user}@{host}/{path}"))
}

fn from_url(url: url::Url, opts: &ParseOptions) -> Result<Addr, ParseError> {
    let scheme = Scheme::parse(url.scheme());

    let userinfo = if !url.username().is_empty() || url.password().is_some() {
        Some(Userinfo {
            username: percent_decode(url.username()),
            password: url.password().map(percent_decode),
        })
    } else {
        None
    };

    let host = match url.host() {
        Some(url::Host::Domain(d)) => Host::Domain(normalize_domain(d)?),
        Some(url::Host::Ipv4(ip)) => Host::Ipv4(ip),
        Some(url::Host::Ipv6(ip)) => Host::Ipv6(ip),
        None if opts.require_host => return Err(ParseError::MissingHost),
        // Schemes like `mailto:` / `file:` / `tel:` have no authority.
        None => Host::Domain(String::new()),
    };

    Ok(Addr {
        scheme,
        userinfo,
        host,
        port: url.port(),
        path: url.path().to_string(),
        query: url.query().map(str::to_string),
        fragment: url.fragment().map(str::to_string),
    })
}

fn percent_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .into_owned()
}

/// For using idna, it's necessary to make sure the domain is not already percent-encoded, otherwise
/// idna will mangle it.
///
/// If `idna` feature is enabled, we ensure the URL is decoded (no-op on already decoded domain), so
/// idna can't get an encoded domain.
#[cfg(feature = "idna")]
fn normalize_domain(d: &str) -> Result<String, ParseError> {
    let decoded = percent_decode(d);
    idna::domain_to_ascii(&decoded)
        .map_err(|e| ParseError::InvalidHost(crate::HostError::BadDomain(e.to_string())))
}

#[cfg(not(feature = "idna"))]
fn normalize_domain(d: &str) -> Result<String, ParseError> {
    Ok(d.to_string())
}

#[cfg(feature = "idna")]
mod tests {
    #[cfg(feature = "idna")]
    use super::{Addr, Host};
    #[test]
    #[cfg(feature = "idna")]
    fn idna_punycodes_nonspecial_scheme() {
        let a = Addr::parse("ssh://git@münchen.de/repo").unwrap();
        assert!(matches!(a.host, Host::Domain(ref d) if d == "xn--mnchen-3ya.de"));
    }

    #[test]
    #[cfg(feature = "idna")]
    fn idna_idempotent_on_special_scheme() {
        let a = Addr::parse("https://münchen.de/").unwrap();
        assert!(matches!(a.host, Host::Domain(ref d) if d == "xn--mnchen-3ya.de"));
    }

    #[test]
    #[cfg(feature = "idna")]
    fn idna_not_used_for_ip6_with_scheme() {
        let a = Addr::parse("ssh://[fe80::1]/x").unwrap();
        assert!(matches!(a.host, Host::Ipv6(_)));
    }
}
