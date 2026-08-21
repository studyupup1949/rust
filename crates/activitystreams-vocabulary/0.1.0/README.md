# ActivityStreams Vocabulary 

This is a library for federating software with the [ActivityPub](https://github.com/w3c/activitypub) protocol.

It implements the [ActivityStreams 2.0 Vocabulary](https://www.w3.org/TR/activitystreams-vocabulary) specification used to define common ActivityPub data structures.

## Example (simple [Object](crate::Object))

```rust
use activitystreams_vocabulary::{Iri, Name, Object};

# fn main() {
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
# }
```

## Example ([Tombstone](crate::Tombstone) in an [OrderedCollection](crate::OrderedCollection))

```rust
use activitystreams_vocabulary::{Deleted, Image, Iri, Item, Name, OrderedCollection, Tombstone};

# fn main() {
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

assert_eq!(serde_json::to_string_pretty(&collection).unwrap(), json_str);
assert_eq!(
    serde_json::from_str::<OrderedCollection>(json_str.as_str()).unwrap(),
    collection
);
# }
```
