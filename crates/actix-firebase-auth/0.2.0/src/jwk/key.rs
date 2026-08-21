use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KeyResponse {
    pub(crate) keys: Vec<JwkKey>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct JwkKey {
    pub(crate) e: String,
    pub(crate) alg: String,
    pub(crate) kty: String,
    pub(crate) kid: String,
    pub(crate) n: String,
}

#[derive(Debug, Clone)]
pub struct JwkKeys {
    pub(crate) keys: Vec<JwkKey>,
    pub(crate) max_age: Duration,
}
