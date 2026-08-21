//! Collection of `Object` and `Object`-derived vocabulary types.
//!
//! ## Defining a Custom Object
//!
//! Library users can define an external `Object`-derived type:
//!
//! ```rust
//! use activitystreams_vocabulary::{ActivityVocabulary, create_object, field_access};
//!
//! // Create a custom `Object` type that inherits all of the base properties.
//! create_object! {
//!     /// Externally created object.
//!     ExternalObject: external_vocab::ExternalType::TestObject {
//!         // Define a custom field.
//!         // Currently, all custom fields must be wrapped with an `Option`.
//!         custom_field: Option<u8>,
//!     }
//! }
//!
//! field_access! {
//!     ExternalObject<Vocab> {
//!         /// Defines access functions for the `custom_field`.
//!         custom_field: option { u8 },
//!     }
//! }
//!
//! # use activitystreams_vocabulary::Context;
//! # use external_vocab::ExternalType;
//! # fn main() {
//! let object = ExternalObject::<ExternalType>::new();
//!
//! // all Object types have the following fields
//! //   (along with `set_`, `with_`, and `unset_` access functions)
//! assert_eq!(object.context_property(), Some(&Context::new()));
//! assert_eq!(object.kind(), &ExternalType::TestObject);
//! assert!(object.id().is_none());
//! assert!(object.name().is_none());
//! assert!(object.name_map().is_none());
//! assert!(object.attachment().is_none());
//! assert!(object.attributed_to().is_none());
//! assert!(object.audience().is_none());
//! assert!(object.summary().is_none());
//! assert!(object.summary_map().is_none());
//! assert!(object.content().is_none());
//! assert!(object.content_map().is_none());
//! assert!(object.context().is_none());
//! assert!(object.generator().is_none());
//! assert!(object.icon().is_none());
//! assert!(object.image().is_none());
//! assert!(object.in_reply_to().is_none());
//! assert!(object.location().is_none());
//! assert!(object.url().is_none());
//! assert!(object.preview().is_none());
//! assert!(object.replies().is_none());
//! assert!(object.tag().is_none());
//! assert!(object.to().is_none());
//! assert!(object.bto().is_none());
//! assert!(object.cc().is_none());
//! assert!(object.bcc().is_none());
//! assert!(object.media_type().is_none());
//! assert!(object.start_time().is_none());
//! assert!(object.end_time().is_none());
//! assert!(object.published().is_none());
//! assert!(object.updated().is_none());
//! assert!(object.duration().is_none());
//! assert!(object.source().is_none());
//! assert!(object.proof().is_none());
//! # }
//! ```
//!
//! For details about the `external_vocab` crate, see the [top-level documentation](crate).

use serde::{Deserialize, Serialize};

use crate::{
    Error, Iri, Result, create_object, impl_default, impl_display, impl_into_item, impl_into_items,
};

mod article;
mod audio;
mod collection;
mod document;
mod event;
mod image;
mod note;
mod page;
mod place;
mod profile;
mod relationship;
mod tombstone;
mod video;

pub use article::*;
pub use audio::*;
pub use collection::*;
pub use document::*;
pub use event::*;
pub use image::*;
pub use note::*;
pub use page::*;
pub use place::*;
pub use profile::*;
pub use relationship::*;
pub use tombstone::*;
pub use video::*;

create_object! {
    /// Represents an ActivityStream [Object](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-object).
    ///
    /// Describes an object of any kind.
    ///
    /// The `Object` type serves as the base type for most of the other kinds of objects defined in the
    /// Activity Vocabulary, including other Core types such as `Activity`, `IntransitiveActivity`,
    /// `Collection`, and `OrderedCollection`.
    ///
    /// # Example (simple)
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, Name, Object};
    ///
    /// # fn main() {
    /// let id = Iri::try_from("http://www.test.example/object/1").unwrap();
    /// let name = Name::try_from("A Simple, non-specific object").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Object",
    ///   "id": "{id}",
    ///   "name": "{name}"
    /// }}"#
    ///     );
    ///
    /// let object = Object::new().with_id(id).with_name(name);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
    ///     object
    /// );
    /// # }
    /// ```
    ///
    /// # Example (with `nameMap` + `attributedTo`)
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, LanguageTag, Link, Name, NameMap, Object};
    ///
    /// # fn main() {
    /// let id = Iri::try_from("http://www.test.example/object/1").unwrap();
    ///
    /// let name_key = LanguageTag::try_from("en").unwrap();
    /// let name_val = Name::try_from("A Simple, non-specific object").unwrap();
    ///
    /// let attributed_name = Name::try_from("Object attribution").unwrap();
    /// let attributed_id = Iri::try_from("http://www.test.example/object/2").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Object",
    ///   "id": "{id}",
    ///   "nameMap": {{
    ///     "{name_key}": "{name_val}"
    ///   }},
    ///   "attributedTo": {{
    ///     "type": "Object",
    ///     "id": "{attributed_id}",
    ///     "name": "{attributed_name}"
    ///   }}
    /// }}"#
    ///     );
    ///
    /// let name_map = NameMap::new().with_map([(name_key, name_val)]);
    ///
    /// let attributed_to = Object::new_inner()
    ///     .with_name(attributed_name.clone())
    ///     .with_id(attributed_id.clone());
    ///
    /// let object = Object::new()
    ///     .with_id(id)
    ///     .with_name_map(name_map)
    ///     .with_attributed_to(attributed_to);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
    ///     object
    /// );
    /// # }
    /// ```
    ///
    /// # Example (with `summaryMap`)
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, LanguageTag, LanguageMap, Name, Object};
    ///
    /// # fn main() {
    /// let id = Iri::try_from("http://www.test.example/object/1").unwrap();
    /// let name = Name::try_from("A Simple, non-specific object").unwrap();
    ///
    /// let summary_key = LanguageTag::try_from("en").unwrap();
    /// let summary_val = "A Simple, <em>non-specific</em> object";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Object",
    ///   "id": "{id}",
    ///   "name": "{name}",
    ///   "summaryMap": {{
    ///     "{summary_key}": "{summary_val}"
    ///   }}
    /// }}"#
    ///     );
    ///
    /// let summary_map = LanguageMap::new().with_map([(summary_key, summary_val.to_string())]);
    ///
    /// let object = Object::new()
    ///     .with_id(id)
    ///     .with_name(name)
    ///     .with_summary_map(summary_map);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
    ///     object
    /// );
    /// # }
    /// ```
    base Object: crate::CoreType::Object {}
}

impl_into_item!(Object, object);
impl_into_items!(Object);

/// Represents an `Object | List<Object> | Iri` range.
///
/// Useful when an external schema is used to define an [Object].
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Objects {
    Single(Box<Object>),
    List(Box<Vec<Object>>),
    Iri(Box<Iri>),
}

impl Objects {
    /// Creates a new [Objects].
    pub fn new() -> Self {
        Self::Single(Box::default())
    }

    /// Creates a new [Objects] [Single](Self::Single) variant.
    pub fn single<I: Into<Object>>(val: I) -> Self {
        Self::Single(Box::new(val.into()))
    }

    /// Gets whether the [Objects] contains a [Single](Self::Single) variant.
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Attempts to get a reference to the [Single](Self::Single) variant.
    pub fn as_single(&self) -> Result<&Object> {
        match self {
            Self::Single(object) => Ok(object),
            _ => Err(Error::object("invalid objects type")),
        }
    }

    /// Attempts to convert to a [Single](Self::Single) variant.
    pub fn to_single(self) -> Result<Object> {
        match self {
            Self::Single(object) => Ok(*object),
            _ => Err(Error::object("invalid objects type")),
        }
    }

    /// Creates a new [Objects] [List](Self::List) variant.
    pub fn list<I: IntoIterator<Item = Object>>(val: I) -> Self {
        Self::List(Box::new(val.into_iter().collect()))
    }

    /// Gets whether the [Objects] contains a [List](Self::List) variant.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Attempts to get a reference to the [List](Self::List) variant.
    pub fn as_list(&self) -> Result<&[Object]> {
        match self {
            Self::List(object) => Ok(object.as_ref()),
            _ => Err(Error::object("invalid objects type")),
        }
    }

    /// Attempts to convert to a [List](Self::List) variant.
    pub fn to_list(self) -> Result<Vec<Object>> {
        match self {
            Self::List(object) => Ok(*object),
            _ => Err(Error::object("invalid objects type")),
        }
    }

    /// Creates a new [Objects] [Iri](Self::Iri) variant.
    pub fn iri<I: Into<Iri>>(val: I) -> Self {
        Self::Iri(Box::new(val.into()))
    }

    /// Gets whether the [Objects] contains a [Iri](Self::Iri) variant.
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to get a reference to the [Iri](Self::Iri) variant.
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(iri) => Ok(iri),
            _ => Err(Error::object("invalid object type")),
        }
    }

    /// Attempts to convert to an [Iri](Self::Iri) variant.
    pub fn to_iri(self) -> Result<Iri> {
        match self {
            Self::Iri(iri) => Ok(*iri),
            _ => Err(Error::object("invalid object type")),
        }
    }
}

impl_default!(Objects);
impl_display!(Objects, json);

impl From<Object> for Objects {
    fn from(val: Object) -> Self {
        Self::Single(Box::new(val))
    }
}

impl<'a> TryFrom<&'a Objects> for &'a Object {
    type Error = Error;

    fn try_from(val: &'a Objects) -> Result<Self> {
        val.as_single()
    }
}

impl TryFrom<Objects> for Object {
    type Error = Error;

    fn try_from(val: Objects) -> Result<Self> {
        val.to_single()
    }
}

impl From<Vec<Object>> for Objects {
    fn from(val: Vec<Object>) -> Self {
        Self::list(val)
    }
}

impl From<&[Object]> for Objects {
    fn from(val: &[Object]) -> Self {
        Self::List(Box::new(val.to_vec()))
    }
}

impl<const N: usize> From<&[Object; N]> for Objects {
    fn from(val: &[Object; N]) -> Self {
        Self::List(Box::new(val.iter().cloned().collect()))
    }
}

impl<const N: usize> From<[Object; N]> for Objects {
    fn from(val: [Object; N]) -> Self {
        Self::list(val)
    }
}

impl<'a> TryFrom<&'a Objects> for &'a [Object] {
    type Error = Error;

    fn try_from(val: &'a Objects) -> Result<Self> {
        val.as_list()
    }
}

impl TryFrom<Objects> for Vec<Object> {
    type Error = Error;

    fn try_from(val: Objects) -> Result<Self> {
        val.to_list()
    }
}

impl From<Iri> for Objects {
    fn from(val: Iri) -> Self {
        Self::Iri(Box::new(val))
    }
}

impl<'a> TryFrom<&'a Objects> for &'a Iri {
    type Error = Error;

    fn try_from(val: &'a Objects) -> Result<Self> {
        val.as_iri()
    }
}

impl TryFrom<Objects> for Iri {
    type Error = Error;

    fn try_from(val: Objects) -> Result<Self> {
        val.to_iri()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Content, Iri, LanguageMap, LanguageTag, Link, MimeType, Name, NameMap};

    #[test]
    fn test_object() {
        let id = Iri::try_from("http://www.test.example/object/1").unwrap();
        let name = Name::try_from("A Simple, non-specific object").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object",
  "id": "{id}",
  "name": "{name}"
}}"#
        );

        let object = Object::new().with_id(id).with_name(name);

        assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            object
        );
    }

    #[test]
    fn test_object_with_link_item() {
        let id = Iri::try_from("http://www.test.example/object/1").unwrap();
        let name = Name::try_from("A Simple, non-specific object").unwrap();
        let href = Iri::try_from("http://example.org/abc").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object",
  "id": "{id}",
  "name": "{name}",
  "attributedTo": {{
    "type": "Link",
    "href": "{href}"
  }}
}}"#
        );

        let attributed_to = Link::new_inner().with_href(href);

        let object = Object::new()
            .with_id(id)
            .with_name(name)
            .with_attributed_to(attributed_to);

        assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            object
        );
    }

    #[test]
    fn test_object_with_object_item() {
        let id = Iri::try_from("http://www.test.example/object/1").unwrap();
        let name = Name::try_from("A Simple, non-specific object").unwrap();

        let attributed_name = Name::try_from("Object attribution").unwrap();
        let attributed_id = Iri::try_from("http://www.test.example/object/2").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object",
  "id": "{id}",
  "name": "{name}",
  "attributedTo": {{
    "type": "Object",
    "id": "{attributed_id}",
    "name": "{attributed_name}"
  }}
}}"#
        );

        let attributed_to = Object::new_inner()
            .with_name(attributed_name)
            .with_id(attributed_id);

        let object = Object::new()
            .with_id(id)
            .with_name(name)
            .with_attributed_to(attributed_to);

        assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            object
        );
    }

    #[test]
    fn test_object_with_name_map() {
        let id = Iri::try_from("http://www.test.example/object/1").unwrap();

        let name_key = LanguageTag::try_from("en").unwrap();
        let name_val = Name::try_from("A Simple, non-specific object").unwrap();

        let attributed_name = Name::try_from("Object attribution").unwrap();
        let attributed_id = Iri::try_from("http://www.test.example/object/2").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object",
  "id": "{id}",
  "nameMap": {{
    "{name_key}": "{name_val}"
  }},
  "attributedTo": {{
    "type": "Object",
    "id": "{attributed_id}",
    "name": "{attributed_name}"
  }}
}}"#
        );

        let name_map = NameMap::new().with_map([(name_key, name_val)]);

        let attributed_to = Object::new_inner()
            .with_name(attributed_name.clone())
            .with_id(attributed_id.clone());

        let object = Object::new()
            .with_id(id)
            .with_name_map(name_map)
            .with_attributed_to(attributed_to);

        assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            object
        );
    }

    #[test]
    fn test_object_with_content_map() {
        let id = Iri::try_from("http://www.test.example/object/1").unwrap();
        let name = Name::try_from("A Simple, non-specific object").unwrap();

        let content_key = LanguageTag::try_from("en").unwrap();
        let content_val = "A Simple, <em>non-specific</em> object";

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object",
  "id": "{id}",
  "name": "{name}",
  "contentMap": {{
    "{content_key}": "{content_val}"
  }}
}}"#
        );

        let content_map = LanguageMap::new().with_map([(content_key, content_val.to_string())]);

        let object = Object::new()
            .with_id(id)
            .with_name(name)
            .with_content_map(content_map);

        assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            object
        );
    }

    #[test]
    fn test_object_with_summary_map() {
        let id = Iri::try_from("http://www.test.example/object/1").unwrap();
        let name = Name::try_from("A Simple, non-specific object").unwrap();

        let summary_key = LanguageTag::try_from("en").unwrap();
        let summary_val = "A Simple, <em>non-specific</em> object";

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object",
  "id": "{id}",
  "name": "{name}",
  "summaryMap": {{
    "{summary_key}": "{summary_val}"
  }}
}}"#
        );

        let summary_map = LanguageMap::new().with_map([(summary_key, summary_val.to_string())]);

        let object = Object::new()
            .with_id(id)
            .with_name(name)
            .with_summary_map(summary_map);

        assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            object
        );
    }

    #[test]
    fn test_object_with_source() {
        let id = Iri::try_from("http://www.test.example/object/1").unwrap();
        let name = Name::try_from("A Simple, non-specific object").unwrap();

        let source_content = "some text";
        let source_type = MimeType::TextPlain;

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object",
  "id": "{id}",
  "name": "{name}",
  "source": {{
    "content": "{source_content}",
    "mediaType": "{source_type}"
  }}
}}"#
        );

        let source = Content::new()
            .with_content(source_content)
            .with_media_type(source_type);

        let object = Object::new()
            .with_id(id.clone())
            .with_name(name.clone())
            .with_source(source);

        assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            object
        );

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Object",
  "id": "{id}",
  "name": "{name}",
  "source": "{source_content}"
}}"#
        );

        let object = Object::new()
            .with_id(id)
            .with_name(name)
            .with_source(source_content);

        assert_eq!(serde_json::to_string_pretty(&object).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            object
        );
    }
}
