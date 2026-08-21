use serde::{Deserialize, Serialize};

use crate::{
    CoreType, Error, Iri, Items, Link, Result, create_object, derived_kind_serde, field_access,
    impl_default, impl_display, impl_into_object,
};

mod ordered;
mod page;

pub use ordered::{OrderedCollection, OrderedCollectionPage, OrderedCollectionPageItem};
pub use page::{CollectionPage, CollectionPageItem};

create_object! {
    /// Represents the ActivityStream [Collection](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-collection) type.
    ///
    /// Represents ordered or unordered sets of [Object](crate::Object)s or [Link]s.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Collection, Name, Note};
    ///
    /// # fn main() {
    /// let summary = "Sally's notes";
    /// let note0_name = Name::try_from("A Simple Note").unwrap();
    /// let note1_name = Name::try_from("Another Simple Note").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Collection",
    ///   "summary": "{summary}",
    ///   "totalItems": 2,
    ///   "items": [
    ///     {{
    ///       "type": "Note",
    ///       "name": "{note0_name}"
    ///     }},
    ///     {{
    ///       "type": "Note",
    ///       "name": "{note1_name}"
    ///     }}
    ///   ]
    /// }}"#);
    ///
    /// let items = [note0_name, note1_name].map(|n| Note::new_inner().with_name(n));
    ///
    /// let collection = Collection::new()
    ///     .with_summary(summary)
    ///     .with_total_items(items.len() as u64)
    ///     .with_items(items);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&collection).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Collection>(json_str.as_str()).unwrap(),
    ///     collection
    /// );
    /// # }
    /// ```
    Collection:
        #[serde(serialize_with = "obj_serde::ser")]
        CoreType {
        #[serde(skip_serializing_if = "Option::is_none")]
        total_items: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        current: Option<Box<CollectionPage>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        first: Option<Box<CollectionPageItem>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        last: Option<Box<CollectionPageItem>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        items: Option<Items>,
    }
}

derived_kind_serde!(crate::CoreType::Collection);
impl_into_object!(Collection);

field_access! {
    Collection {
        /// A non-negative integer specifying the total number of objects contained by the logical view of the collection.
        ///
        /// This number might not reflect the actual number of items serialized within the [Collection] object instance.
        total_items: option { u64 },
    }
}

field_access! {
    Collection {
        /// Identifies the items contained in a collection.
        ///
        /// The items might be ordered or unordered.
        items: option_ref { Items },
    }
}

field_access! {
    Collection {
        /// In a paged [Collection], indicates the page that contains the most recently updated member items.
        current: option_box_deref { CollectionPage },
        /// In a paged [Collection], indicates the furthest preceeding page of items in the collection.
        first: option_box_deref { CollectionPageItem },
        /// In a paged [Collection], indicates the furthest proceeding page of the collection.
        last: option_box_deref { CollectionPageItem },
    }
}

/// Represents the ActivityStream `Collection | Link` range.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CollectionItem {
    Collection(Box<Collection>),
    Link(Box<Link>),
    Iri(Box<Iri>),
}

impl CollectionItem {
    /// Creates a new [CollectionItem].
    pub fn new() -> Self {
        Self::Collection(Box::default())
    }

    /// Gets whether the [CollectionItem] is an [Collection] variant.
    #[inline]
    pub const fn is_collection(&self) -> bool {
        matches!(self, Self::Collection(_))
    }

    /// Attempts to convert the [CollectionItem] to an [Collection].
    pub fn as_collection(&self) -> Result<&Collection> {
        match self {
            Self::Collection(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item collection type: {ty}"))),
        }
    }

    /// Gets whether the [CollectionItem] is an [Link] variant.
    #[inline]
    pub const fn is_link(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Attempts to convert the [CollectionItem] to an [Link].
    pub fn as_link(&self) -> Result<&Link> {
        match self {
            Self::Link(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item link type: {ty}"))),
        }
    }

    /// Gets whether the [CollectionItem] is an [Iri] variant.
    #[inline]
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to convert the [CollectionItem] to an [Iri].
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item iri type: {ty}"))),
        }
    }
}

impl_default!(CollectionItem);
impl_display!(CollectionItem, json);

impl From<Collection> for CollectionItem {
    fn from(val: Collection) -> Self {
        Self::Collection(Box::new(val))
    }
}

impl From<Link> for CollectionItem {
    fn from(val: Link) -> Self {
        Self::Link(Box::new(val))
    }
}

impl From<Iri> for CollectionItem {
    fn from(val: Iri) -> Self {
        Self::Iri(Box::new(val))
    }
}

impl<'a> TryFrom<&'a CollectionItem> for &'a Collection {
    type Error = Error;

    fn try_from(val: &'a CollectionItem) -> Result<Self> {
        val.as_collection()
    }
}

impl<'a> TryFrom<&'a CollectionItem> for &'a Link {
    type Error = Error;

    fn try_from(val: &'a CollectionItem) -> Result<Self> {
        val.as_link()
    }
}

impl<'a> TryFrom<&'a CollectionItem> for &'a Iri {
    type Error = Error;

    fn try_from(val: &'a CollectionItem) -> Result<Self> {
        val.as_iri()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Note};

    #[test]
    fn test_collection() {
        let summary = "Sally's notes";
        let note0_name = Name::try_from("A Simple Note").unwrap();
        let note1_name = Name::try_from("Another Simple Note").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Collection",
  "summary": "{summary}",
  "totalItems": 2,
  "items": [
    {{
      "type": "Note",
      "name": "{note0_name}"
    }},
    {{
      "type": "Note",
      "name": "{note1_name}"
    }}
  ]
}}"#
        );

        let items = [note0_name, note1_name].map(|n| Note::new_inner().with_name(n));

        let collection = Collection::new()
            .with_summary(summary)
            .with_total_items(items.len() as u64)
            .with_items(items);

        assert_eq!(serde_json::to_string_pretty(&collection).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Collection>(json_str.as_str()).unwrap(),
            collection
        );
    }

    #[test]
    fn test_invalid_collection() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Collection>(json_str).is_err());
    }
}
