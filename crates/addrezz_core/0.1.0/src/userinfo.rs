use core::fmt;

/// Userinfo subcomponent of the authority (RFC 3986 §3.2.1).
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Userinfo {
    /// Login username
    pub username: String,
    /// Optional login password
    pub password: Option<String>,
}

impl Userinfo {
    /// Create userinfo with just a username.
    pub fn new(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: None,
        }
    }

    /// Set the password via builder pattern.
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }
}

impl fmt::Display for Userinfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.password {
            Some(p) => write!(f, "{}:{}", self.username, p),
            None => f.write_str(&self.username),
        }
    }
}

// Redacting Debug prevents password leaks via `{:?}` when the `redact`
// feature is on. Display still emits the real value because that is the
// wire format.
#[cfg(feature = "redact")]
impl fmt::Debug for Userinfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Userinfo")
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "***"))
            .finish()
    }
}

#[cfg(not(feature = "redact"))]
impl fmt::Debug for Userinfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Userinfo")
            .field("username", &self.username)
            .field("password", &self.password)
            .finish()
    }
}
