use hyper::Uri;

use serde::{Deserialize, Serialize, de, ser};

use crate::{Error, Link, Result, impl_default, impl_display, impl_into_item, impl_into_items};

/// Represents an ActivityStream IRI (international resource identifier).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Iri(String);

impl Iri {
    /// Creates a new [Iri].
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Convenience method to create an [Iri] from a known-valid IRI string.
    pub(crate) const fn new_trusted(iri: String) -> Self {
        Self(iri)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets the string representation of the [Iri].
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts the [Iri] to a URI.
    pub fn uri(&self) -> Result<Uri> {
        Uri::try_from(self.0.as_str()).map_err(|err| Error::iri(err.to_string()))
    }

    /// Converts an URI into an [Iri].
    pub fn from_uri<U: Into<Uri>>(val: U) -> Self {
        Self(format!("{}", val.into()))
    }
}

impl_default!(Iri);
impl_display!(Iri, str);
impl_into_item!(Iri, iri);
impl_into_items!(Iri);

impl TryFrom<String> for Iri {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<&str> for Iri {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        val.parse::<Uri>()
            .map(|_| Iri(val.into()))
            .map_err(|err| Error::Iri(format!("{err}")))
    }
}

impl ser::Serialize for Iri {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for Iri {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer)
            .and_then(|s| {
                s.parse()
                    .map_err(|err| de::Error::custom(format!("invalid IRI: {err}")))
            })
            .map(Iri)
    }
}

/// Represents the ActivityStream range `anyURI | Link`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum IriItem {
    Iri(Box<Iri>),
    Link(Box<Link>),
    Links(Box<Vec<Link>>),
}

impl IriItem {
    /// Creates a new [IriItem].
    #[inline]
    pub fn new() -> Self {
        Self::Iri(Box::default())
    }

    /// Creates a new [IriItem] [Link](Self::Link) variant.
    #[inline]
    pub fn link<I: Into<Link>>(link: I) -> Self {
        Self::Link(Box::new(link.into()))
    }

    /// Gets whether the [IriItem] is an [Link] variant.
    #[inline]
    pub const fn is_link(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Attempts to convert the [IriItem] to an [Link].
    #[inline]
    pub fn as_link(&self) -> Result<&Link> {
        match self {
            Self::Link(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item link type: {ty}"))),
        }
    }

    /// Creates a new [IriItem] [Links](Self::Links) variant.
    #[inline]
    pub fn links<I: IntoIterator<Item = Link>>(link: I) -> Self {
        Self::Links(Box::new(link.into_iter().collect()))
    }

    /// Gets whether the [IriItem] is an [Links](Self::Links) variant.
    #[inline]
    pub const fn is_links(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Attempts to convert the [IriItem] to an [Links](Self::Links).
    #[inline]
    pub fn as_links(&self) -> Result<&[Link]> {
        match self {
            Self::Links(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item link type: {ty}"))),
        }
    }

    /// Creates a new [IriItem] [Iri](Self::Iri) variant.
    #[inline]
    pub fn iri<I: Into<Iri>>(iri: I) -> Self {
        Self::Iri(Box::new(iri.into()))
    }

    /// Gets whether the [IriItem] is an [Iri] variant.
    #[inline]
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to convert the [IriItem] to an [Iri].
    #[inline]
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item iri type: {ty}"))),
        }
    }
}

impl From<Iri> for IriItem {
    fn from(val: Iri) -> Self {
        Self::Iri(Box::new(val))
    }
}

impl From<Link> for IriItem {
    fn from(val: Link) -> Self {
        Self::Link(Box::new(val))
    }
}

impl From<Vec<Link>> for IriItem {
    fn from(val: Vec<Link>) -> Self {
        Self::Links(Box::new(val))
    }
}

impl From<&[Link]> for IriItem {
    fn from(val: &[Link]) -> Self {
        Self::Links(Box::new(val.into_iter().cloned().collect()))
    }
}

impl<const N: usize> From<&[Link; N]> for IriItem {
    fn from(val: &[Link; N]) -> Self {
        Self::Links(Box::new(val.into_iter().cloned().collect()))
    }
}

impl<const N: usize> From<[Link; N]> for IriItem {
    fn from(val: [Link; N]) -> Self {
        Self::Links(Box::new(val.into_iter().collect()))
    }
}

impl<'a> TryFrom<&'a IriItem> for &'a Iri {
    type Error = Error;

    fn try_from(val: &'a IriItem) -> Result<Self> {
        val.as_iri()
    }
}

impl<'a> TryFrom<&'a IriItem> for &'a Link {
    type Error = Error;

    fn try_from(val: &'a IriItem) -> Result<Self> {
        val.as_link()
    }
}

impl_default!(IriItem);
impl_display!(IriItem, json);
