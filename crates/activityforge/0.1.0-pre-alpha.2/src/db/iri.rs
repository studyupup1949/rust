use activitystreams_vocabulary::{Iri as VocabIri, Item, LinkItem, impl_default, impl_display};
use http::Uri;
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

    /// Extracts the base IRI scheme, host, and (optional) port.
    pub fn base_iri(&self) -> Result<Self> {
        let uri = Uri::try_from(self.as_str())
            .map_err(|err| Error::db(format!("iri: invalid URI: {err}")))?;

        let scheme = uri.scheme_str().ok_or(Error::db("iri: missing scheme"))?;

        let host = uri.host().ok_or(Error::db("iri: missing host"))?;

        if let Some(port) = uri.port() {
            Ok(Self(format!("{scheme}://{host}:{port}")))
        } else {
            Ok(Self(format!("{scheme}://{host}")))
        }
    }
}

impl_default!(Iri);
impl_display!(Iri, str);

impl From<&Iri> for Iri {
    fn from(val: &Self) -> Self {
        val.clone()
    }
}

impl From<VocabIri> for Iri {
    fn from(val: VocabIri) -> Self {
        (&val).into()
    }
}

impl From<&VocabIri> for Iri {
    fn from(val: &VocabIri) -> Self {
        Self(val.to_string())
    }
}

impl From<Option<VocabIri>> for Iri {
    fn from(val: Option<VocabIri>) -> Self {
        val.as_ref().into()
    }
}

impl From<Option<&VocabIri>> for Iri {
    fn from(val: Option<&VocabIri>) -> Self {
        val.map(|i| Self(i.to_string())).unwrap_or_default()
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

impl From<Item> for Iri {
    fn from(val: Item) -> Self {
        (&val).into()
    }
}

impl From<&Item> for Iri {
    fn from(val: &Item) -> Self {
        match val {
            Item::Object(obj) => obj.id().map(|i| i.into()).unwrap_or_default(),
            Item::Link(link) => link.href().into(),
            Item::Iri(iri) => iri.as_ref().into(),
        }
    }
}

impl From<LinkItem> for Iri {
    fn from(val: LinkItem) -> Self {
        (&val).into()
    }
}

impl From<&LinkItem> for Iri {
    fn from(val: &LinkItem) -> Self {
        match val {
            LinkItem::Link(link) => link.href().into(),
            LinkItem::Iri(iri) => iri.as_ref().into(),
        }
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
