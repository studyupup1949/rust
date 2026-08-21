use serde::{Deserialize, Serialize};

use crate::{
    CoreType, Error, Iri, Link, OrderedItems, Result, create_object, derived_kind_serde,
    field_access, impl_default, impl_display, impl_into_object,
};

use super::OrderedCollectionItem;

create_object! {
    /// Represents the ActivityStream [OrderedCollectionPage](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-collectionpage) type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, Name, Note, OrderedCollectionPage};
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
    ///   "type": "OrderedCollectionPage",
    ///   "id": "{id}",
    ///   "summary": "{summary}",
    ///   "partOf": "{part_of}",
    ///   "orderedItems": [
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
    /// let collection_page = OrderedCollectionPage::new()
    ///     .with_id(id)
    ///     .with_summary(summary)
    ///     .with_part_of(part_of)
    ///     .with_ordered_items(items);
    ///
    /// assert_eq!(
    ///     serde_json::to_string_pretty(&collection_page).unwrap(),
    ///     json_str
    /// );
    /// assert_eq!(
    ///     serde_json::from_str::<OrderedCollectionPage>(json_str.as_str()).unwrap(),
    ///     collection_page,
    /// );
    /// # }
    /// ```
    OrderedCollectionPage:
        #[serde(serialize_with = "obj_serde::ser")]
        CoreType {
        #[serde(skip_serializing_if = "Option::is_none")]
        part_of: Option<OrderedCollectionItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        total_items: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        current: Option<Box<OrderedCollectionPage>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        first: Option<Box<OrderedCollectionPageItem>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        last: Option<Box<OrderedCollectionPageItem>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ordered_items: Option<OrderedItems>,
        #[serde(skip_serializing_if = "Option::is_none")]
        next: Option<Box<OrderedCollectionPageItem>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        prev: Option<Box<OrderedCollectionPageItem>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_index: Option<usize>,
    }
}

derived_kind_serde!(crate::CoreType::OrderedCollectionPage);
impl_into_object!(OrderedCollectionPage);

field_access! {
    OrderedCollectionPage {
        /// A non-negative integer specifying the total number of objects contained by the logical view of the collection.
        ///
        /// This number might not reflect the actual number of items serialized within the [Collection](crate::Collection) object instance.
        total_items: option { u64 },
        /// A non-negative integer value identifying the relative position within the logical view of a strictly ordered collection.
        start_index: option { usize },
    }
}

field_access! {
    OrderedCollectionPage {
        /// Identifies the items contained in a collection.
        ///
        /// The items might be ordered or unordered.
        ordered_items: option_ref { OrderedItems },
        /// Identifies the [Collection](crate::Collection) to which a [OrderedCollectionPage] objects items belong.
        part_of: option_ref { OrderedCollectionItem },
    }
}

field_access! {
    OrderedCollectionPage {
        /// In a paged [Collection](crate::Collection), indicates the page that contains the most recently updated member items.
        current: option_box_deref { OrderedCollectionPage },
        /// In a paged [Collection](crate::Collection), indicates the furthest preceeding page of items in the collection.
        first: option_box_deref { OrderedCollectionPageItem },
        /// In a paged [Collection](crate::Collection), indicates the furthest proceeding page of the collection.
        last: option_box_deref { OrderedCollectionPageItem },
    }
}

field_access! {
    OrderedCollectionPage {
        /// In a paged [Collection](crate::Collection), indicates the next page of items.
        next: option_box_deref { OrderedCollectionPageItem },
        /// In a paged [Collection](crate::Collection), indicates the previous page of items.
        prev: option_box_deref { OrderedCollectionPageItem },
    }
}

/// Represents the ActivityStream `OrderedCollectionPage | Link` range.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OrderedCollectionPageItem {
    OrderedCollectionPage(Box<OrderedCollectionPage>),
    Link(Box<Link>),
    Iri(Box<Iri>),
}

impl OrderedCollectionPageItem {
    /// Creates a new [OrderedCollectionPageItem].
    pub fn new() -> Self {
        Self::OrderedCollectionPage(Box::default())
    }

    /// Gets whether the [OrderedCollectionPageItem] is an [OrderedCollectionPage] variant.
    #[inline]
    pub const fn is_collection_page(&self) -> bool {
        matches!(self, Self::OrderedCollectionPage(_))
    }

    /// Attempts to convert the [OrderedCollectionPageItem] to an [OrderedCollectionPage].
    pub fn as_collection_page(&self) -> Result<&OrderedCollectionPage> {
        match self {
            Self::OrderedCollectionPage(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!(
                "invalid item collection page type: {ty}"
            ))),
        }
    }

    /// Gets whether the [OrderedCollectionPageItem] is an [Link] variant.
    #[inline]
    pub const fn is_link(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Attempts to convert the [OrderedCollectionPageItem] to an [Link].
    pub fn as_link(&self) -> Result<&Link> {
        match self {
            Self::Link(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item link type: {ty}"))),
        }
    }

    /// Gets whether the [OrderedCollectionPageItem] is an [Iri] variant.
    #[inline]
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to convert the [OrderedCollectionPageItem] to an [Iri].
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(ty) => Ok(ty.as_ref()),
            ty => Err(Error::item(format!("invalid item iri type: {ty}"))),
        }
    }
}

impl_default!(OrderedCollectionPageItem);
impl_display!(OrderedCollectionPageItem, json);

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
  "type": "OrderedCollectionPage",
  "id": "{id}",
  "summary": "{summary}",
  "partOf": "{part_of}",
  "orderedItems": [
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

        let collection_page = OrderedCollectionPage::new()
            .with_id(id)
            .with_summary(summary)
            .with_part_of(part_of)
            .with_ordered_items(items);

        assert_eq!(
            serde_json::to_string_pretty(&collection_page).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<OrderedCollectionPage>(json_str.as_str()).unwrap(),
            collection_page,
        );
    }

    #[test]
    fn test_invalid_collection_page() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<OrderedCollectionPage>(json_str).is_err());
    }
}
