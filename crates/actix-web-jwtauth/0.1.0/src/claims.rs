use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct Jwt<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    aud: Option<HashSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exp: Option<u64>,
    // Token payload
    payload: T,
}

impl<T> std::ops::Deref for Jwt<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

fn seconds_since_unix_epoch() -> u64 {
    std::time::UNIX_EPOCH
        .elapsed()
        .expect("Time went backwards")
        .as_secs()
}

impl<T> Jwt<T> {
    pub fn with(payload: T) -> Self {
        Self {
            aud: None,
            sub: None,
            exp: None,
            payload,
        }
    }

    /// Add `audience`
    pub fn audience<S: ToString>(mut self, audience: S) -> Self {
        match self.aud {
            Some(ref mut xs) => {
                xs.insert(audience.to_string());
            }

            None => {
                let mut xs = HashSet::with_capacity(1);
                xs.insert(audience.to_string());
                self.aud = Some(xs);
            }
        }

        self
    }

    // Set `subject`
    pub fn subject<S: ToString>(mut self, subject: S) -> Self {
        self.sub.replace(subject.to_string());
        self
    }

    /// Expires in some `seconds`
    pub fn expires(mut self, expires: u64) -> Self {
        self.exp.replace(seconds_since_unix_epoch() + expires);
        self
    }
}
