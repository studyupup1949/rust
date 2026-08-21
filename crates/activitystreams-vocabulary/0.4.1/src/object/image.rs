use serde::{Deserialize, Serialize};

use crate::{Error, Iri, Link, Result, create_object, impl_default, impl_display};

create_object! {
    /// An image document of any kind.
    Image: ObjectType {}
}

/// Represents the ActivityStream `Image | Link` range.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ImageItem {
    Image(Box<Image>),
    Link(Box<Link>),
    Links(Box<Vec<Link>>),
    Iri(Box<Iri>),
    Iris(Box<Vec<Iri>>),
}

impl ImageItem {
    /// Creates a new [ImageItem].
    pub fn new() -> Self {
        Self::Image(Box::default())
    }

    /// Gets whether the [ImageItem] is an [Image] variant.
    #[inline]
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Image(_))
    }

    /// Attempts to convert the [ImageItem] to an [Image].
    pub fn as_object(&self) -> Result<&Image> {
        match self {
            Self::Image(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item image type: {ty}"))),
        }
    }

    /// Gets whether the [ImageItem] is an [Link] variant.
    #[inline]
    pub const fn is_link(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Attempts to convert the [ImageItem] to an [Link].
    pub fn as_link(&self) -> Result<&Link> {
        match self {
            Self::Link(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item link type: {ty}"))),
        }
    }

    /// Creates a new [ImageItem] [Links](Self::Links) variant.
    #[inline]
    pub fn links<I: IntoIterator<Item = Link>>(link: I) -> Self {
        Self::Links(Box::new(link.into_iter().collect()))
    }

    /// Gets whether the [ImageItem] is an [Links](Self::Links) variant.
    #[inline]
    pub const fn is_links(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Attempts to convert the [ImageItem] to an [Links](Self::Links).
    #[inline]
    pub fn as_links(&self) -> Result<&[Link]> {
        match self {
            Self::Links(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item link type: {ty}"))),
        }
    }

    /// Gets whether the [ImageItem] is an [Iri] variant.
    #[inline]
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to convert the [ImageItem] to an [Iri].
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item iri type: {ty}"))),
        }
    }

    /// Creates a new [ImageItem] [Iris](Self::Iris) variant.
    #[inline]
    pub fn iris<I: IntoIterator<Item = Iri>>(link: I) -> Self {
        Self::Iris(Box::new(link.into_iter().collect()))
    }

    /// Gets whether the [ImageItem] is an [Iris](Self::Iris) variant.
    #[inline]
    pub const fn is_iris(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to convert the [ImageItem] to an [Iris](Self::Iris).
    #[inline]
    pub fn as_iris(&self) -> Result<&[Iri]> {
        match self {
            Self::Iris(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item link type: {ty}"))),
        }
    }
}

impl_default!(ImageItem);
impl_display!(ImageItem, json);

impl From<Iri> for ImageItem {
    fn from(val: Iri) -> Self {
        Self::Iri(Box::new(val))
    }
}

impl From<Link> for ImageItem {
    fn from(val: Link) -> Self {
        Self::Link(Box::new(val))
    }
}

impl From<Vec<Link>> for ImageItem {
    fn from(val: Vec<Link>) -> Self {
        Self::Links(Box::new(val))
    }
}

impl From<&[Link]> for ImageItem {
    fn from(val: &[Link]) -> Self {
        Self::Links(Box::new(val.to_vec()))
    }
}

impl<const N: usize> From<&[Link; N]> for ImageItem {
    fn from(val: &[Link; N]) -> Self {
        Self::Links(Box::new(val.iter().cloned().collect()))
    }
}

impl<const N: usize> From<[Link; N]> for ImageItem {
    fn from(val: [Link; N]) -> Self {
        Self::Links(Box::new(val.into_iter().collect()))
    }
}

impl From<Vec<Iri>> for ImageItem {
    fn from(val: Vec<Iri>) -> Self {
        Self::Iris(Box::new(val))
    }
}

impl From<&[Iri]> for ImageItem {
    fn from(val: &[Iri]) -> Self {
        Self::Iris(Box::new(val.to_vec()))
    }
}

impl<const N: usize> From<&[Iri; N]> for ImageItem {
    fn from(val: &[Iri; N]) -> Self {
        Self::Iris(Box::new(val.iter().cloned().collect()))
    }
}

impl<const N: usize> From<[Iri; N]> for ImageItem {
    fn from(val: [Iri; N]) -> Self {
        Self::Iris(Box::new(val.to_vec()))
    }
}

impl<'a> TryFrom<&'a ImageItem> for &'a Iri {
    type Error = Error;

    fn try_from(val: &'a ImageItem) -> Result<Self> {
        val.as_iri()
    }
}

impl<'a> TryFrom<&'a ImageItem> for &'a Link {
    type Error = Error;

    fn try_from(val: &'a ImageItem) -> Result<Self> {
        val.as_link()
    }
}

impl<'a> TryFrom<&'a ImageItem> for &'a [Link] {
    type Error = Error;

    fn try_from(val: &'a ImageItem) -> Result<Self> {
        val.as_links()
    }
}

impl<'a> TryFrom<&'a ImageItem> for &'a [Iri] {
    type Error = Error;

    fn try_from(val: &'a ImageItem) -> Result<Self> {
        val.as_iris()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, MimeType, Name};

    #[test]
    fn test_image() {
        let name = Name::try_from("Cat Jumping on Wagon").unwrap();

        let url0_href = Iri::try_from("http://example.org/image.jpeg").unwrap();
        let url0_mime = MimeType::ImageJpeg;

        let url1_href = Iri::try_from("http://example.org/image.png").unwrap();
        let url1_mime = MimeType::ImagePng;

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Image",
  "name": "{name}",
  "url": [
    {{
      "type": "Link",
      "href": "{url0_href}",
      "mediaType": "{url0_mime}"
    }},
    {{
      "type": "Link",
      "href": "{url1_href}",
      "mediaType": "{url1_mime}"
    }}
  ]
}}"#
        );

        let urls = [(url0_href, url0_mime), (url1_href, url1_mime)]
            .map(|(href, mime)| Link::new_inner().with_href(href).with_media_type(mime));

        let image = Image::new().with_name(name).with_url(urls);

        assert_eq!(serde_json::to_string_pretty(&image).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Image>(json_str.as_str()).unwrap(),
            image
        );
    }

    #[test]
    fn test_invalid_image() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Image>(json_str).is_err());
    }
}
