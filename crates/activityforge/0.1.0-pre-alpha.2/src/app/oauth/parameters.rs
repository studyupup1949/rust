use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Represents a collection of OAuth-2.0 token request normalized parameters.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct NormalizedParameter(HashMap<String, String>);

impl NormalizedParameter {
    /// Creates a new [NormalizedParameter].
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts a new entry into the list.
    pub fn insert<K, V>(&mut self, k: K, v: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.0.insert(k.into(), v.into());
    }
}

impl std::fmt::Display for NormalizedParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, (k, v)) in self.0.iter().enumerate() {
            if i == 0 {
                write!(f, "{k}={v}")?;
            } else {
                write!(f, "&{k}={v}")?;
            }
        }

        std::fmt::Result::Ok(())
    }
}
