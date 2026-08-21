# ActivityStreams Vocabulary 

This is a library for federating software with the [ActivityPub](https://github.com/w3c/activitypub) protocol.

It implements the [ActivityStreams 2.0 Vocabulary](https://www.w3.org/TR/activitystreams-vocabulary) specification used to define common ActivityPub data structures.

## Alternatives

- [activitystreams-kinds](https://docs.rs/activitystreams-kinds)
  - provides a bare-bones approach to ActivityStreams Vocabulary types
  - only provides "kind" types, and not the associated data structures
- [go-ap/activitypub](https://pkg.go.dev/github.com/go-ap/activitypub)
  - mature Go implementation of ActivityPub data structures
  - influenced a number of design decisions in `activitystreams-vocabulary`
    - [ActivityVocabulary](https://docs.rs/activitystreams-vocabulary/latest/activitystreams_vocabulary/trait.ActivityVocabulary.html) is based on [Typer](https://pkg.go.dev/github.com/go-ap/activitypub#Typer)
    - [VocabularyType](https://docs.rs/activitystreams-vocabulary/latest/activitystreams_vocabulary/enum.VocabularyType.html) is based on [ActivityVocabularyType](https://pkg.go.dev/github.com/go-ap/activitypub#ActivityVocabularyType)
    - [VocabularyTypes](https://docs.rs/activitystreams-vocabulary/latest/activitystreams_vocabulary/enum.VocabularyTypes.html) is based on [ActivityVocabularyTypes](https://pkg.go.dev/github.com/go-ap/activitypub#ActivityVocabularyTypes)

## Using the Base Vocabulary Types

`activitystreams-vocabulary` aims to implement as many base vocabulary types as specified in common ActivityPub documents.

This includes types from the base [`ActivityStreams 2.0 Vocabulary`](https://www.w3.org/TR/activitystreams-vocabulary), and commonly implemented [`FEP` extensions](https://codeberg.org/fediverse/fep).

### Constructing a Simple [Object](crate::Object)

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

### Using a [Tombstone](crate::Tombstone) in an [OrderedCollection](crate::OrderedCollection)

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

## Extending the Base Vocabulary

`ActivityPub` and `ActivityStreams` are both intended to be extendable protocols, and so this crate was designed to follow that lead.

A number of helper macros exist to help create types that build on the base vocabulary.

### Creating a Derived `Object` Type

For the following examples, we're going to assume types are defined in a crate named is `external_vocab`.

*hint*: `external-vocab` is defined in [tests/external-vocab](https://codeberg.org/activitypub-rs/activitystreams-vocabulary/src/branch/main/tests/external-vocab).

Defining the crate in `tests/external-vocab` helps with automated testing of external usage, and working within the constraints of Rust `doc-tests`.

#### Defining a Custom Vocabulary Type

First, let's define the `type` field enum `ExternalType`:

```rust
use serde::{Deserialize, Serialize};

use activitystreams_vocabulary::{impl_activity_vocabulary, impl_display, impl_into_vocabulary};

/// Represents the extended vocabulary types for your new types.
///
/// These variants will encode to the `type` field of the new type.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub enum ExternalType {
    #[default]
    TestObject,
    TestActor,
}

impl ExternalType {
    /// Gets the [ExternalType] string representation.
    ///
    /// Required for the helper macro implementations of traits.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TestObject => "TestObject",
            Self::TestActor => "TestActor",
        }
    }
}

// Convenience macro to implement the `Display` trait.
impl_display!(ExternalType, str);
// Implement the `ActivityVocabulary` trait for interoperability with the `activitystreams_vocabulary` base types.
impl_activity_vocabulary!(ExternalType);
// Implement the `From` trait to convert `ExternalType` into `VocabularyType` + `VocabularyTypes`.
impl_into_vocabulary!(ExternalType);
```

**NOTE**: The vocabulary type can be defined anywhere in the crate, but should be re-exported from the crate root. This is mostly needed because of limitations in current helper macro implementations.

#### Defining a Custom Object

Next, we can define an external `Object`-derived type:

```rust
use activitystreams_vocabulary::{ActivityVocabulary, create_actor, create_object, field_access};

// Create a custom `Object` type that inherits all of the base properties.
create_object! {
    /// Externally created object.
    ExternalObject: external_vocab::ExternalType::TestObject {
        // Define a custom field.
        // Currently, all custom fields must be wrapped with an `Option`.
        custom_field: Option<u8>,
    }
}

field_access! {
    ExternalObject {
        /// Defines access functions for the `custom_field`.
        custom_field: option { u8 },
    }
}
```

#### Defining a Custom Actor

Defining a custom `Actor`-derived type inherits all of the properties of an `Object`, along with all of the `Actor` properties.

To define a custom `Actor`-derived type:

```rust
use activitystreams_vocabulary::create_actor;

// Create a custom `Actor` type that inherits all of the base `Actor` + `Object` properties.
create_actor! {
    /// Externally created actor.
    ExternalActor:
        external_vocab::ExternalType::TestActor {}
}
```

Additional fields can be defined for custom `Actor` types similar to `Object`s:

```rust
use activitystreams_vocabulary::{create_actor, field_access};

// Create a custom `Actor` type that inherits all of the base `Actor` + `Object` properties.
create_actor! {
    /// Externally created actor.
    ExternalActor:
        external_vocab::ExternalType::TestActor {
        custom_field1: Option<usize>,
        custom_field2: Option<u8>,
        string_field: Option<String>,
        vec_field: Option<Vec<u8>>,
   }
}

// Field access definitions need to be grouped based on the access type, e.g. `option`, `option_deref`, etc.
field_access! {
    ExternalActor {
        custom_field1: option { usize },
        custom_field2: option { u8 },
    }
}

// `option_deref` uses the `Option::as_deref` function to get a reference to the `Deref` type, e.g. `Option<&str>` for `Option<String>`.
field_access! {
    ExternalActor {
        string_field: option_deref { &str, String },
        vec_field: option_deref { &[u8], Vec<u8> },
    }
}
```

## Design Tradeoffs

Even with heavy macro use, there is still a bit of boilerplate when creating new types (more helper macros to come :).

However, there is an enormous amount of boilerplate avoided by taking advantage of Rust's excellent macro infrastructure.

Future iterations may also use `proc-macros` which are even more powerful than the `declarative-macros` currently in use. 

The choice to use `declarative-macros` is mostly due to not requiring an external `macros`-crate, and the reduced number of dependencies.

## Fuck AI

This application was made with 100% human engineering, entirely without the aid of LLMs.
