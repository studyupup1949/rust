//! Collection of `Link` and `Link`-derived types.
//!
//! # Creating a Custom Link
//!
//! ```rust
//! use activitystreams_vocabulary::create_link;
//!
//! create_link! {
//!     /// Externally created link.
//!     ExternalLink: external_vocab::ExternalType::TestLink {}
//! }
//!
//! # use activitystreams_vocabulary::{Context, Iri};
//! # use external_vocab::ExternalType;
//! # fn main() {
//! let link = ExternalLink::<ExternalType>::new();
//! // all Activity types have the following fields
//! //   (along with `set_`, `with_`, and `unset_` access functions)
//! assert_eq!(link.context_property(), Some(&Context::new()));
//! assert_eq!(link.kind(), &ExternalType::TestLink);
//! assert_eq!(link.href(), &Iri::new());
//! assert!(link.name().is_none());
//! assert!(link.name_map().is_none());
//! assert!(link.rel().is_none());
//! assert!(link.media_type().is_none());
//! assert!(link.hreflang().is_none());
//! assert!(link.preview().is_none());
//! assert!(link.height().is_none());
//! assert!(link.width().is_none());
//! # }
//! ```
//!
//! For details about the `external_vocab` crate, see the [top-level documentation](crate).

use serde::{Deserialize, Serialize};

use crate::{Error, Result, create_link, impl_default, impl_display};

mod mention;

pub use mention::*;

create_link! {
    /// Represents an ActivityStream [Link](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-link).
    ///
    /// A `Link` is an indirect, qualified reference to a resource identified by a URL.
    ///
    /// The fundamental model for links is established by [RFC5988](https://www.rfc-editor.org/rfc/rfc5988).
    ///
    /// Many of the properties defined by the Activity Vocabulary allow values that are either instances of
    /// `Object` or `Link`.
    ///
    /// When a `Link` is used, it establishes a qualified relation connecting the subject (the containing object)
    /// to the resource identified by the `href`.
    ///
    /// Properties of the Link are properties of the reference as opposed to properties of the resource.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, LanguageTag, Link, MimeType, Name};
    ///
    /// # fn main() {
    /// let name = Name::try_from("An example link").unwrap();
    /// let href = Iri::try_from("http://example.org/abc").unwrap();
    /// let hreflang = LanguageTag::try_from("en").unwrap();
    /// let media_type = MimeType::TextHtml;
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Link",
    ///   "href": "{href}",
    ///   "name": "{name}",
    ///   "hreflang": "{hreflang}",
    ///   "mediaType": "{media_type}"
    /// }}"#
    ///         );
    ///
    /// let link = Link::new()
    ///     .with_href(href)
    ///     .with_hreflang(hreflang)
    ///     .with_name(name)
    ///     .with_media_type(media_type);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&link).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Link>(json_str.as_str()).unwrap(),
    ///     link
    /// );
    /// # }
    /// ```
    base Link:
        #[serde(serialize_with = "obj_serde::ser")]
        crate::CoreType::Link {}
}

/// Represents the ActivityStream
/// [Links](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-link) type.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Links {
    Single(Box<Link>),
    List(Box<Vec<Link>>),
}

impl Links {
    /// Creates a new [Links].
    pub fn new() -> Self {
        Self::Single(Box::default())
    }

    /// Creates an [Links] [Single](Self::Single) variant.
    pub fn single<I: Into<Link>>(val: I) -> Self {
        Self::Single(Box::new(val.into()))
    }

    /// Gets whether the [Links] contains a [Single](Self::Single) variant.
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Attempts to get a reference to the [Single](Self::Single) variant.
    pub fn as_single(&self) -> Result<&Link> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::link("invalid link type")),
        }
    }

    /// Attempts to convert to the [Single](Self::Single) variant.
    pub fn to_single(self) -> Result<Link> {
        match self {
            Self::Single(ty) => Ok(*ty),
            _ => Err(Error::link("invalid link type")),
        }
    }

    /// Creates an [Links] [List](Self::List) variant.
    pub fn list<T, I>(val: I) -> Self
    where
        T: Into<Link>,
        I: IntoIterator<Item = T>,
    {
        Self::List(Box::new(val.into_iter().map(|i| i.into()).collect()))
    }

    /// Gets whether the [Links] contains a [List](Self::List) variant.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Attempts to get a reference to the [List](Self::List) variant.
    pub fn as_list(&self) -> Result<&[Link]> {
        match self {
            Self::List(tys) => Ok(tys),
            _ => Err(Error::link("invalid link type")),
        }
    }

    /// Attempts to convert to the [List](Self::List) variant.
    pub fn to_list(self) -> Result<Vec<Link>> {
        match self {
            Self::List(tys) => Ok(*tys),
            _ => Err(Error::link("invalid link type")),
        }
    }
}

impl_default!(Links);
impl_display!(Links, json);

impl<I: Into<Link>> From<I> for Links {
    fn from(val: I) -> Self {
        Self::single(val)
    }
}

impl<I: Into<Link>> From<Vec<I>> for Links {
    fn from(val: Vec<I>) -> Self {
        Self::list(val)
    }
}

impl<I: Into<Link> + Clone> From<&[I]> for Links {
    fn from(val: &[I]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<I: Into<Link> + Clone, const N: usize> From<&[I; N]> for Links {
    fn from(val: &[I; N]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<I: Into<Link>, const N: usize> From<[I; N]> for Links {
    fn from(val: [I; N]) -> Self {
        Self::list(val)
    }
}

impl TryFrom<Links> for Link {
    type Error = Error;

    fn try_from(val: Links) -> Result<Self> {
        val.to_single()
    }
}

impl<'a> TryFrom<&'a Links> for &'a Link {
    type Error = Error;

    fn try_from(val: &'a Links) -> Result<Self> {
        val.as_single()
    }
}

impl TryFrom<Links> for Vec<Link> {
    type Error = Error;

    fn try_from(val: Links) -> Result<Self> {
        val.to_list()
    }
}

impl<'a> TryFrom<&'a Links> for &'a [Link] {
    type Error = Error;

    fn try_from(val: &'a Links) -> Result<Self> {
        val.as_list()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreType, Iri, LanguageTag, MimeType, Name, VocabularyTypes};

    #[test]
    fn test_link_required() {
        let name = Name::try_from("An example link").unwrap();
        let href = Iri::try_from("http://example.org/abc").unwrap();
        let hreflang = LanguageTag::try_from("en").unwrap();
        let media_type = MimeType::TextHtml;

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Link",
  "href": "{href}",
  "name": "{name}",
  "hreflang": "{hreflang}",
  "mediaType": "{media_type}"
}}"#
        );

        let link = Link::new()
            .with_href(href)
            .with_hreflang(hreflang)
            .with_name(name)
            .with_media_type(media_type);

        assert_eq!(serde_json::to_string_pretty(&link).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Link>(json_str.as_str()).unwrap(),
            link
        );
    }

    #[test]
    fn test_link_full() {
        let href = Iri::try_from("http://example.org/abc").unwrap();
        let name = Name::try_from("An example link").unwrap();
        let rel = Iri::try_from("http://exampl.org/abc/relation#test").unwrap();
        let hreflang = LanguageTag::try_from("en").unwrap();
        let media_type = MimeType::TextHtml;
        let height = 1u64;
        let width = 1u64;

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Link",
  "href": "{href}",
  "name": "{name}",
  "rel": "{rel}",
  "hreflang": "{hreflang}",
  "mediaType": "{media_type}",
  "height": {height},
  "width": {width}
}}"#
        );

        let link = Link::new()
            .with_href(href)
            .with_name(name)
            .with_media_type(media_type)
            .with_rel(rel)
            .with_hreflang(hreflang)
            .with_height(height)
            .with_width(width);

        assert_eq!(serde_json::to_string_pretty(&link).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Link>(json_str.as_str()).unwrap(),
            link
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;
        let link = Link::<VocabularyTypes>::new().with_kind(CoreType::Object);

        assert!(serde_json::to_string(&link).is_err());
        assert!(serde_json::from_str::<Link>(json_str).is_err());
    }
}
