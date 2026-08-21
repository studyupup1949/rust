use serde::{Deserialize, Serialize};

use crate::{
    Error, Iri, Link, ObjectType, Result, create_object, derived_kind_serde, impl_default,
    impl_display, impl_into_object,
};

create_object! {
    /// An image document of any kind.
    Image:
        #[serde(serialize_with = "obj_serde::ser")]
        ObjectType {}
}

derived_kind_serde!(crate::ObjectType::Image);
impl_into_object!(Image);

/// Represents the ActivityStream `Image | Link` range.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ImageItem {
    Image(Box<Image>),
    Link(Box<Link>),
    Iri(Box<Iri>),
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
}

impl_default!(ImageItem);
impl_display!(ImageItem, json);

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
