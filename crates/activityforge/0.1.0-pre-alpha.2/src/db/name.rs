use activitystreams_vocabulary::{Name as VocabName, impl_default, impl_display};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Represents a [Name](VocabName) SQL database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct Name(String);

impl Name {
    /// Creates a new [Name].
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Gets whether the [Name] is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets the [Name] length.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets the string representation.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl_default!(Name);
impl_display!(Name, str);

impl From<VocabName> for Name {
    fn from(val: VocabName) -> Self {
        (&val).into()
    }
}

impl From<&VocabName> for Name {
    fn from(val: &VocabName) -> Self {
        Self(val.to_string())
    }
}

impl From<Name> for VocabName {
    fn from(val: Name) -> Self {
        Self::try_from(val.0).unwrap_or_default()
    }
}

impl From<&Name> for VocabName {
    fn from(val: &Name) -> Self {
        Self::try_from(val.as_str()).unwrap_or_default()
    }
}

impl TryFrom<String> for Name {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        VocabName::try_from(val.as_str())
            .map(Self::from)
            .map_err(Error::from)
    }
}

impl TryFrom<&String> for Name {
    type Error = Error;

    fn try_from(val: &String) -> Result<Self> {
        VocabName::try_from(val.as_str())
            .map(Self::from)
            .map_err(Error::from)
    }
}

impl TryFrom<&str> for Name {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        VocabName::try_from(val)
            .map(Self::from)
            .map_err(Error::from)
    }
}
