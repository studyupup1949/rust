use serde::{Deserialize, Serialize};

use crate::{Error, MimeType, Result, field_access, impl_default, impl_display};

/// Represents the [source property](https://www.w3.org/TR/activitypub#source-property) of an ActivityPub [Object](crate::Object).
///
/// In addition to all the properties defined by the [Activity-Vocabulary](https://www.w3.org/TR/activitystreams-vocabulary), ActivityPub extends the [Object](crate::Object) by supplying the source property.
///
/// The `source` property is intended to convey some sort of source from which the content markup was derived, as a form of provenance, or to support future editing by clients.
///
/// In general, clients do the conversion from source to content, not the other way around.
///
/// The value of `source` is itself an object which uses its own `content` and `mediaType` fields to supply source information.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    content: String,
    media_type: MimeType,
}

impl Content {
    /// Creates a new [Content].
    pub const fn new() -> Self {
        Self {
            content: String::new(),
            media_type: MimeType::new(),
        }
    }
}

field_access! {
    Content {
        /// Represents the source text content.
        content: as_ref { &str, String },
    }
}

field_access! {
    Content {
        /// Represents the source media type.
        media_type: MimeType,
    }
}

impl_default!(Content);
impl_display!(Content, json);

/// Represents a [Content] object or flattened-[`String`] representation.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ContentItem {
    Object(Content),
    Flat(String),
}

impl ContentItem {
    /// Creates a new [ContentItem].
    pub const fn new() -> Self {
        Self::Object(Content::new())
    }

    /// Creates a new [ContentItem] [Object](Self::Object) variant.
    pub fn object<I: Into<Content>>(val: I) -> Self {
        Self::Object(val.into())
    }

    /// Gets whether the [ContentItem] is an [Object](Self::Object) variant.
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Converts the [ContentItem] into a [Content].
    pub fn as_object(&self) -> Result<&Content> {
        match self {
            Self::Object(ty) => Ok(ty),
            _ => Err(Error::source("invalid source object variant")),
        }
    }

    /// Converts the [ContentItem] into a [Content].
    pub fn to_object(self) -> Result<Content> {
        match self {
            Self::Object(ty) => Ok(ty),
            _ => Err(Error::source("invalid source object variant")),
        }
    }

    /// Creates a new [ContentItem] [Flat](Self::Flat) variant.
    pub fn flat<I: Into<String>>(val: I) -> Self {
        Self::Flat(val.into())
    }

    /// Gets whether the [ContentItem] is an [Flat](Self::Flat) variant.
    pub const fn is_flat(&self) -> bool {
        matches!(self, Self::Flat(_))
    }

    /// Converts the [ContentItem] into a string.
    pub fn as_flat(&self) -> Result<&str> {
        match self {
            Self::Flat(ty) => Ok(ty),
            _ => Err(Error::source("invalid source flat variant")),
        }
    }

    /// Converts the [ContentItem] into a string.
    pub fn to_flat(self) -> Result<String> {
        match self {
            Self::Flat(ty) => Ok(ty),
            _ => Err(Error::source("invalid source flat variant")),
        }
    }
}

impl_default!(ContentItem);
impl_display!(ContentItem, json);

impl From<Content> for ContentItem {
    fn from(val: Content) -> Self {
        Self::object(val)
    }
}

impl<I: Into<String>> From<I> for ContentItem {
    fn from(val: I) -> Self {
        Self::flat(val)
    }
}

impl<'a> TryFrom<&'a ContentItem> for &'a str {
    type Error = Error;

    fn try_from(val: &'a ContentItem) -> Result<Self> {
        val.as_flat()
    }
}

impl TryFrom<&ContentItem> for String {
    type Error = Error;

    fn try_from(val: &ContentItem) -> Result<Self> {
        val.as_flat().map(|i| i.to_string())
    }
}

impl TryFrom<ContentItem> for String {
    type Error = Error;

    fn try_from(val: ContentItem) -> Result<Self> {
        val.to_flat()
    }
}

impl<'a> TryFrom<&'a ContentItem> for &'a Content {
    type Error = Error;

    fn try_from(val: &'a ContentItem) -> Result<Self> {
        val.as_object()
    }
}

impl TryFrom<&ContentItem> for Content {
    type Error = Error;

    fn try_from(val: &ContentItem) -> Result<Self> {
        val.as_object().cloned()
    }
}

impl TryFrom<ContentItem> for Content {
    type Error = Error;

    fn try_from(val: ContentItem) -> Result<Self> {
        val.to_object()
    }
}
