use serde::{Deserialize, Serialize};

use crate::{
    DateTime, Error, Objects, Result, create_object, derived_kind_serde, field_access,
    impl_default, impl_display, impl_into_object,
};

create_object! {
    /// Represents an audio document of any kind.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Deleted, Iri, Tombstone};
    ///
    /// # fn main() {
    ///     let former_type = Iri::try_from("Image").unwrap();
    ///     let id = Iri::try_from("http://image.example/2").unwrap();
    ///     let deleted = Deleted::try_from("2016-03-17T00:00:00Z").unwrap();
    ///
    ///     let json_str = format!(
    ///         r#"{{
    ///   "type": "Tombstone",
    ///   "id": "{id}",
    ///   "deleted": {deleted},
    ///   "formerType": "{former_type}"
    /// }}"#
    ///     );
    ///
    /// let tombstone = Tombstone::new_inner()
    ///     .with_id(id)
    ///     .with_former_type(former_type)
    ///     .with_deleted(deleted);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&tombstone).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Tombstone>(json_str.as_str()).unwrap(),
    ///     tombstone
    /// );
    /// # }
    /// ```
    ///
    /// # Example (in an [OrderedCollection](crate::OrderedCollection))
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Deleted, Image, Iri, Item, Name, OrderedCollection, Tombstone};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Vacation photos 2016").unwrap();
    /// let former_type = Iri::try_from("Image").unwrap();
    /// let tombstone_id = Iri::try_from("http://image.example/2").unwrap();
    /// let deleted = Deleted::try_from("2016-03-17T00:00:00Z").unwrap();
    /// let image1_id = Iri::try_from("http://image.example/1").unwrap();
    /// let image3_id = Iri::try_from("http://image.example/3").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "OrderedCollection",
    ///   "name": "{name}",
    ///   "totalItems": 3,
    ///   "orderedItems": [
    ///     {{
    ///       "type": "Image",
    ///       "id": "{image1_id}"
    ///     }},
    ///     {{
    ///       "type": "Tombstone",
    ///       "id": "{tombstone_id}",
    ///       "deleted": {deleted},
    ///       "formerType": "{former_type}"
    ///     }},
    ///     {{
    ///       "type": "Image",
    ///       "id": "{image3_id}"
    ///     }}
    ///   ]
    /// }}"#
    ///     );
    ///
    /// let image1 = Image::new_inner().with_id(image1_id);
    /// let image3 = Image::new_inner().with_id(image3_id);
    /// let tombstone = Tombstone::new_inner()
    ///     .with_id(tombstone_id)
    ///     .with_former_type(former_type)
    ///     .with_deleted(deleted);
    ///
    /// let items = [
    ///     Item::from(image1),
    ///     Item::from(tombstone),
    ///     Item::from(image3),
    /// ];
    ///
    /// let collection = OrderedCollection::new()
    ///     .with_name(name)
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
    Tombstone:
        #[serde(serialize_with = "obj_serde::ser")]
        ObjectType {
            #[serde(skip_serializing_if = "Option::is_none")]
            deleted: Option<Deleted>,
            #[serde(skip_serializing_if = "Option::is_none")]
            former_type: Option<Box<Objects>>,
        }
}

derived_kind_serde!(crate::ObjectType::Tombstone);
impl_into_object!(Tombstone {
    deleted,
    former_type,
});

field_access! {
    Tombstone {
        /// On a [Tombstone] object, the `formerType` property identifies the type of the object that was deleted.
        former_type: option_box_deref { Objects },
    }
}

field_access! {
    Tombstone {
        /// On a [Tombstone] object, the `deleted` property is a timestamp for when the object was deleted.
        deleted: option { Deleted },
    }
}

/// On a [Tombstone] object, the `deleted` property is a timestamp for when the object was deleted.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Deleted(DateTime);

impl Deleted {
    /// Creates a new [Deleted].
    pub fn new() -> Self {
        Self(DateTime::default())
    }

    /// Creates a new [Deleted] [DateTime](crate::DateTime).
    pub fn from_date_time<E, I>(val: I) -> Result<Self>
    where
        E: core::error::Error,
        I: TryInto<DateTime, Error = E>,
    {
        val.try_into()
            .map_err(|err| Error::tombstone(format!("invalid deleted date-time: {err}")))
            .map(Self)
    }
}

impl core::convert::AsRef<DateTime> for Deleted {
    fn as_ref(&self) -> &DateTime {
        &self.0
    }
}

impl_default!(Deleted);
impl_display!(Deleted, json);

impl core::str::FromStr for Deleted {
    type Err = Error;

    /// Creates a new [Deleted] [DateTime](crate::DateTime) from a string.
    fn from_str(val: &str) -> Result<Self> {
        val.parse::<DateTime>()
            .map_err(|err| Error::tombstone(format!("invalid deleted date-time: {err}")))
            .map(Self)
    }
}

impl TryFrom<&str> for Deleted {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        use core::str::FromStr;

        Self::from_str(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Image, Iri, Item, Name, OrderedCollection};

    #[test]
    fn test_valid() {
        let former_type = Iri::try_from("Image").unwrap();
        let id = Iri::try_from("http://image.example/2").unwrap();
        let deleted = Deleted::try_from("2016-03-17T00:00:00Z").unwrap();

        let json_str = format!(
            r#"{{
  "type": "Tombstone",
  "id": "{id}",
  "deleted": {deleted},
  "formerType": "{former_type}"
}}"#
        );

        let tombstone = Tombstone::new_inner()
            .with_id(id)
            .with_former_type(former_type)
            .with_deleted(deleted);

        assert_eq!(serde_json::to_string_pretty(&tombstone).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Tombstone>(json_str.as_str()).unwrap(),
            tombstone
        );
    }

    #[test]
    fn test_valid_collection() {
        let name = Name::try_from("Vacation photos 2016").unwrap();
        let former_type = Iri::try_from("Image").unwrap();
        let tombstone_id = Iri::try_from("http://image.example/2").unwrap();
        let deleted = Deleted::try_from("2016-03-17T00:00:00Z").unwrap();
        let image1_id = Iri::try_from("http://image.example/1").unwrap();
        let image3_id = Iri::try_from("http://image.example/3").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "OrderedCollection",
  "name": "{name}",
  "totalItems": 3,
  "orderedItems": [
    {{
      "type": "Image",
      "id": "{image1_id}"
    }},
    {{
      "type": "Tombstone",
      "id": "{tombstone_id}",
      "deleted": {deleted},
      "formerType": "{former_type}"
    }},
    {{
      "type": "Image",
      "id": "{image3_id}"
    }}
  ]
}}"#
        );

        let image1 = Image::new_inner().with_id(image1_id);
        let image3 = Image::new_inner().with_id(image3_id);
        let tombstone = Tombstone::new_inner()
            .with_id(tombstone_id)
            .with_former_type(former_type)
            .with_deleted(deleted);

        let items = [
            Item::from(image1),
            Item::from(tombstone),
            Item::from(image3),
        ];

        let collection = OrderedCollection::new()
            .with_name(name)
            .with_total_items(items.len() as u64)
            .with_ordered_items(items);

        println!(
            "{}\n{}",
            serde_json::to_string_pretty(&collection).unwrap(),
            json_str
        );
        assert_eq!(serde_json::to_string_pretty(&collection).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<OrderedCollection>(json_str.as_str()).unwrap(),
            collection
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Tombstone>(json_str).is_err());
    }
}
