use activitystreams_vocabulary::{Iri as VocabIri, impl_default, impl_display};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Represents a IRI SQL database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct Iri(String);

impl Iri {
    /// Creates a new [Iri].
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Gets whether the [Iri] is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets the [Iri] length.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets the string representation.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl_default!(Iri);
impl_display!(Iri, str);

impl From<VocabIri> for Iri {
    fn from(val: VocabIri) -> Self {
        Self(val.to_string())
    }
}

impl From<Iri> for VocabIri {
    fn from(val: Iri) -> Self {
        Self::try_from(val.as_str()).unwrap_or_default()
    }
}

impl From<&Iri> for VocabIri {
    fn from(val: &Iri) -> Self {
        Self::try_from(val.as_str()).unwrap_or_default()
    }
}

impl TryFrom<String> for Iri {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        VocabIri::try_from(val.as_str())
            .map(Self::from)
            .map_err(Error::from)
    }
}

impl TryFrom<&String> for Iri {
    type Error = Error;

    fn try_from(val: &String) -> Result<Self> {
        VocabIri::try_from(val.as_str())
            .map(Self::from)
            .map_err(Error::from)
    }
}

impl TryFrom<&str> for Iri {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        VocabIri::try_from(val).map(Self::from).map_err(Error::from)
    }
}
