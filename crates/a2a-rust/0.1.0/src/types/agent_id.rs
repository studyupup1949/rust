use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::A2AError;

/// Validated helper type for agent identifiers used in application-level naming.
///
/// This is not a tagged proto field. It exists to codify the repository's
/// naming convention for agent IDs:
///
/// - only lowercase ASCII letters, digits, and `-`
/// - length between 3 and 64 characters
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct AgentId(String);

impl AgentId {
    /// Validate and construct an `AgentId`.
    pub fn new(value: impl Into<String>) -> Result<Self, A2AError> {
        let value = value.into();
        validate_agent_id(&value)?;
        Ok(Self(value))
    }

    /// Return the validated identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AgentId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<AgentId> for String {
    fn from(value: AgentId) -> Self {
        value.0
    }
}

impl TryFrom<String> for AgentId {
    type Error = A2AError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for AgentId {
    type Error = A2AError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for AgentId {
    type Err = A2AError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl<'de> Deserialize<'de> for AgentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

fn validate_agent_id(value: &str) -> Result<(), A2AError> {
    if !(3..=64).contains(&value.len()) {
        return Err(A2AError::InvalidRequest(
            "agent_id must be between 3 and 64 characters".to_owned(),
        ));
    }

    if !value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(A2AError::InvalidRequest(
            "agent_id may only contain lowercase ASCII letters, digits, and '-'".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::AgentId;

    #[test]
    fn agent_id_accepts_valid_values() {
        let agent_id = AgentId::new("echo-agent-01").expect("agent id should validate");
        assert_eq!(agent_id.as_str(), "echo-agent-01");
    }

    #[test]
    fn agent_id_rejects_invalid_characters() {
        let error = AgentId::new("Echo_Agent").expect_err("agent id should be invalid");
        assert!(
            error
                .to_string()
                .contains("lowercase ASCII letters, digits, and '-'")
        );
    }

    #[test]
    fn agent_id_rejects_invalid_length() {
        let error = AgentId::new("ab").expect_err("agent id should be invalid");
        assert!(error.to_string().contains("between 3 and 64 characters"));
    }
}
