use serde::{Deserialize, Serialize};

use crate::{
    Error, Iri, Items, Link, Result, create_object, derived_kind_serde, field_access, impl_default,
    impl_display, impl_into_object,
};

use super::CollectionItem;

create_object! {
    /// Represents the ActivityStream
    /// [CollectionPage](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-collectionpage) type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{CollectionPage, Iri, Name, Note};
    ///
    /// # fn main() {
    /// let summary = "Page 1 of Sally's notes";
    /// let id = Iri::try_from("http://example.org/foo?page=1").unwrap();
    /// let part_of = Iri::try_from("http://example.org/foo").unwrap();
    /// let note0_name = Name::try_from("A Simple Note").unwrap();
    /// let note1_name = Name::try_from("Another Simple Note").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "CollectionPage",
    ///   "id": "{id}",
    ///   "summary": "{summary}",
    ///   "partOf": "{part_of}",
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
    /// let collection_page = CollectionPage::new()
    ///     .with_summary(summary)
    ///     .with_id(id)
    ///     .with_part_of(part_of)
    ///     .with_items(items);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&collection_page).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<CollectionPage>(json_str.as_str()).unwrap(),
    ///     collection_page
    /// );
    /// # }
    /// ```
    CollectionPage:
        #[serde(deserialize_with = "obj_serde::de", serialize_with = "obj_serde::ser")]
        CoreType {
        #[serde(skip_serializing_if = "Option::is_none")]
        part_of: Option<CollectionItem>,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        next: Option<Box<CollectionPageItem>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        prev: Option<Box<CollectionPageItem>>,
    }
}

derived_kind_serde!(crate::CoreType::CollectionPage);
impl_into_object!(CollectionPage);

field_access! {
    CollectionPage {
        /// A non-negative integer specifying the total number of objects contained by the logical view of the collection.
        ///
        /// This number might not reflect the actual number of items serialized within the [Collection](crate::Collection) object instance.
        total_items: option { u64 },
    }
}

field_access! {
    CollectionPage {
        /// Identifies the items contained in a collection.
        ///
        /// The items might be ordered or unordered.
        items: option_ref { Items },
        /// Identifies the [Collection](crate::Collection) to which a [CollectionPage] objects items belong.
        part_of: option_ref { CollectionItem },
    }
}

field_access! {
    CollectionPage {
        /// In a paged [Collection](crate::Collection), indicates the page that contains the most recently updated member items.
        current: option_box_deref { CollectionPage },
        /// In a paged [Collection](crate::Collection), indicates the furthest preceeding page of items in the collection.
        first: option_box_deref { CollectionPageItem },
        /// In a paged [Collection](crate::Collection), indicates the furthest proceeding page of the collection.
        last: option_box_deref { CollectionPageItem },
    }
}

field_access! {
    CollectionPage {
        /// In a paged [Collection](crate::Collection), indicates the next page of items.
        next: option_box_deref { CollectionPageItem },
        /// In a paged [Collection](crate::Collection), indicates the previous page of items.
        prev: option_box_deref { CollectionPageItem },
    }
}

/// Represents the ActivityStream `CollectionPage | Link` range.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CollectionPageItem {
    CollectionPage(Box<CollectionPage>),
    Link(Box<Link>),
    Iri(Box<Iri>),
}

impl CollectionPageItem {
    /// Creates a new [CollectionPageItem].
    pub fn new() -> Self {
        Self::CollectionPage(Box::default())
    }

    /// Gets whether the [CollectionPageItem] is an [CollectionPage] variant.
    #[inline]
    pub const fn is_collection_page(&self) -> bool {
        matches!(self, Self::CollectionPage(_))
    }

    /// Attempts to convert the [CollectionPageItem] to an [CollectionPage].
    pub fn as_collection_page(&self) -> Result<&CollectionPage> {
        match self {
            Self::CollectionPage(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!(
                "invalid item collection page type: {ty}"
            ))),
        }
    }

    /// Gets whether the [CollectionPageItem] is an [Link] variant.
    #[inline]
    pub const fn is_link(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Attempts to convert the [CollectionPageItem] to an [Link].
    pub fn as_link(&self) -> Result<&Link> {
        match self {
            Self::Link(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item link type: {ty}"))),
        }
    }

    /// Gets whether the [CollectionPageItem] is an [Iri] variant.
    #[inline]
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to convert the [CollectionPageItem] to an [Iri].
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item iri type: {ty}"))),
        }
    }
}

impl_default!(CollectionPageItem);
impl_display!(CollectionPageItem, json);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name, Note};

    #[test]
    fn test_collection_page() {
        let summary = "Page 1 of Sally's notes";
        let id = Iri::try_from("http://example.org/foo?page=1").unwrap();
        let part_of = Iri::try_from("http://example.org/foo").unwrap();
        let note0_name = Name::try_from("A Simple Note").unwrap();
        let note1_name = Name::try_from("Another Simple Note").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "CollectionPage",
  "id": "{id}",
  "summary": "{summary}",
  "partOf": "{part_of}",
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

        let collection_page = CollectionPage::new()
            .with_summary(summary)
            .with_id(id)
            .with_part_of(part_of)
            .with_items(items);

        assert_eq!(
            serde_json::to_string_pretty(&collection_page).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<CollectionPage>(json_str.as_str()).unwrap(),
            collection_page
        );
    }

    #[test]
    fn test_invalid_collection_page() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<CollectionPage>(json_str).is_err());
    }
}
