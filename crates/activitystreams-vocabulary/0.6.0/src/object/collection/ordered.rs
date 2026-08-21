use crate::{Iri, Link, OrderedItems, create_item, create_list, create_object, field_access};

mod page;

pub use page::*;

create_object! {
    /// Represents the ActivityStream [OrderedCollection](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-collection) type.
    ///
    /// Represents ordered or unordered sets of [Object](crate::Object)s or [Link]s.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Name, Note, OrderedCollection};
    ///
    /// # fn main() {
    /// let summary = "Sally's notes";
    /// let note0_name = Name::try_from("A Simple Note").unwrap();
    /// let note1_name = Name::try_from("Another Simple Note").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "OrderedCollection",
    ///   "summary": "{summary}",
    ///   "totalItems": 2,
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
    /// let collection = OrderedCollection::new()
    ///     .with_summary(summary)
    ///     .with_total_items(items.len() as u64)
    ///     .with_ordered_items(items);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&collection).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<OrderedCollection>(json_str.as_str()).unwrap(),
    ///     collection
    /// );
    /// # }
    /// ```
    OrderedCollection: CoreType {
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
    }
}

field_access! {
    OrderedCollection<Vocab> {
        /// A non-negative integer specifying the total number of objects contained by the logical view of the collection.
        ///
        /// This number might not reflect the actual number of items serialized within the [OrderedCollection] object instance.
        total_items: option { u64 },
    }
}

field_access! {
    OrderedCollection<Vocab> {
        /// Identifies the items contained in a collection.
        ///
        /// The items might be ordered or unordered.
        ordered_items: option_ref { OrderedItems },
    }
}

field_access! {
    OrderedCollection<Vocab> {
        /// In a paged [OrderedCollection], indicates the page that contains the most recently updated member items.
        current: option_box_deref { OrderedCollectionPage },
        /// In a paged [OrderedCollection], indicates the furthest preceeding page of items in the collection.
        first: option_box_deref { OrderedCollectionPageItem },
        /// In a paged [OrderedCollection], indicates the furthest proceeding page of the collection.
        last: option_box_deref { OrderedCollectionPageItem },
    }
}

create_item! {
    /// Represents the ActivityStream `OrderedCollection | Link` range.
    OrderedCollectionItem
        boxed
        default: Self::OrderedCollection(Box::new(OrderedCollection::new_inner())),
    {
        OrderedCollection(OrderedCollection),
        Link(Link),
        Iri(Iri),
    }
}

create_list! {
    /// Represents a list of [OrderedCollectionItem]s.
    OrderedCollectionItems: OrderedCollectionItem,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Note, Object};

    #[test]
    fn test_collection() {
        let summary = "Sally's notes";
        let note0_name = Name::try_from("A Simple Note").unwrap();
        let note1_name = Name::try_from("Another Simple Note").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "OrderedCollection",
  "summary": "{summary}",
  "totalItems": 2,
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
        let collection = OrderedCollection::new()
            .with_summary(summary)
            .with_total_items(items.len() as u64)
            .with_ordered_items(items);

        assert_eq!(serde_json::to_string_pretty(&collection).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<OrderedCollection>(json_str.as_str()).unwrap(),
            collection
        );
    }

    #[test]
    fn test_collection_object() {
        let summary = "Sally's notes";
        let note0_name = Name::try_from("A Simple Note").unwrap();
        let note1_name = Name::try_from("Another Simple Note").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "OrderedCollection",
  "summary": "{summary}",
  "orderedItems": [
    {{
      "type": "Note",
      "name": "{note0_name}"
    }},
    {{
      "type": "Note",
      "name": "{note1_name}"
    }}
  ],
  "totalItems": 2
}}"#
        );

        let items = [note0_name, note1_name].map(|n| Note::new_inner().with_name(n));
        let collection: Object = OrderedCollection::new()
            .with_summary(summary)
            .with_total_items(items.len() as u64)
            .with_ordered_items(items)
            .into();

        assert_eq!(serde_json::to_string_pretty(&collection).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            collection
        );
    }

    #[test]
    fn test_invalid_collection() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<OrderedCollection>(json_str).is_err());
    }
}
