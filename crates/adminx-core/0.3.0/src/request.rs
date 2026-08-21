// adminx-core/src/request.rs
//
// A neutral request context. Adapters build this from their framework's request
// (extracting the query string, path params, authenticated claims, and the CSRF
// cookie) and hand it to Resource methods.

use std::collections::HashMap;

/// Authenticated principal, decoded from the auth cookie by `auth::build_ctx`.
#[derive(Debug, Clone, Default)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub roles: Vec<String>,
    /// MFA step for this session: `"ok"` (fully authenticated) or `"pending"`
    /// (password verified, awaiting a second factor). Empty on anonymous ctx.
    pub mfa: String,
}

#[derive(Debug, Clone, Default)]
pub struct ReqCtx {
    /// Raw query string (without leading `?`).
    pub query: String,
    /// Prefix the adminx app is mounted at (e.g. `/adminx`), used to build links.
    pub mount: String,
    pub path_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub claims: Option<Claims>,
    /// Raw CSRF cookie value, if the request carried one. Compared against the
    /// hidden form field on POST; see `crate::csrf`.
    pub csrf: Option<String>,
}

impl ReqCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    pub fn with_mount(mut self, mount: impl Into<String>) -> Self {
        self.mount = mount.into();
        self
    }

    pub fn with_claims(mut self, claims: Claims) -> Self {
        self.claims = Some(claims);
        self
    }

    pub fn with_csrf(mut self, csrf: impl Into<String>) -> Self {
        self.csrf = Some(csrf.into());
        self
    }

    /// All roles for the current principal (`role` plus `roles`), empty if anon.
    pub fn roles(&self) -> Vec<String> {
        match &self.claims {
            Some(c) => {
                let mut r = c.roles.clone();
                if !c.role.is_empty() && !r.contains(&c.role) {
                    r.push(c.role.clone());
                }
                r
            }
            None => Vec::new(),
        }
    }
}
