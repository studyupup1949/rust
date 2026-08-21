use crate::Scheme;

/// Options that steer how ambiguous inputs are resolved.
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Scheme to assume when the input has none (e.g. `gitlab.com`).
    pub default_scheme: Scheme,
    /// Accept SCP-style `user@host:path` as SSH.
    pub allow_scp_like: bool,
    /// Require a non-empty host after parsing.
    pub require_host: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            default_scheme: default_scheme_from_features(),
            allow_scp_like: true,
            require_host: true,
        }
    }
}

impl ParseOptions {
    /// Override the default scheme used for bare hosts.
    pub fn with_default_scheme(mut self, s: Scheme) -> Self {
        self.default_scheme = s;
        self
    }
}

#[inline]
fn default_scheme_from_features() -> Scheme {
    #[cfg(feature = "default_http")]
    {
        Scheme::Http
    }
    #[cfg(not(feature = "default_http"))]
    {
        Scheme::Https
    }
}
