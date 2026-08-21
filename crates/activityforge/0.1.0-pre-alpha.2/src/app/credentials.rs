use activitystreams_vocabulary::{impl_default, impl_display};
use base64::Encoding;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{Error, Result};

/// Represents user credentials for authentication.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Zeroize, ZeroizeOnDrop)]
pub struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    /// Create a new [Credentials].
    pub const fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
        }
    }

    /// Creates a new [Credentials] from the provided parameters.
    pub fn create<U, P>(username: U, password: P) -> Self
    where
        U: Into<String>,
        P: Into<String>,
    {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Gets the username.
    pub fn username(&self) -> &str {
        self.username.as_str()
    }

    /// Gets the password.
    pub fn password(&self) -> &str {
        self.password.as_str()
    }

    /// Attempts to parse [Credentials] encoded in a basic-auth header.
    pub fn from_basic_auth(header: &str) -> Result<Self> {
        header
            .strip_prefix("Basic ")
            .ok_or(Error::http("creds: invalid basic-auth header"))
            .and_then(|c| {
                base64::Base64::decode_vec(c).map_err(|err| {
                    Error::http(format!("creds: error decoding basic-auth header: {err}"))
                })
            })
            .and_then(|c| {
                String::from_utf8(c)
                    .map_err(|err| Error::http(format!("creds: invalid UTF-8 encoding: {err}")))
            })
            .and_then(|c| {
                c.split_once(":")
                    .ok_or(Error::http("creds: missing delimiter"))
                    .map(|(u, p)| Self {
                        username: u.into(),
                        password: p.into(),
                    })
            })
    }

    /// Converts the [Credentials to a basic-auth header value.
    pub fn to_basic_auth(&self) -> String {
        let b64 = base64::Base64::encode_string(
            format!("{}:{}", self.username, self.password).as_bytes(),
        );
        format!("Basic {b64}")
    }
}

impl_default!(Credentials);
impl_display!(Credentials, json);
