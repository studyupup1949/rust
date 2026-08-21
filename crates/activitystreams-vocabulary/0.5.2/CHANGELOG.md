# Releases

## Unreleased

## 0.5.2

- improve `Item` + `Items` conversion traits by providing bi-directional conversion between variants

## 0.5.1

- improve `Object` conversion traits by providing bi-directional conversion between derived objects

## 0.5.0

- **BREAKING CHANGE** move `base` keyword in macro arms
  - moved the `base` keyword for better aesthetics + consistency
- **BREAKING CHANGE** redefines the `MultikeyItem` type with `create_item`
- **BREAKING CHANGE** redefines the `KeyItem` type with `create_item`
- added the `OrderedCollectionPageItems` list type
- **BREAKING CHANGE** redefines the `OrderedCollectionPageItem` type with `create_item`
- added the `CollectionPageItems` list type
- **BREAKING CHANGE** redefines the `CollectionPageItem` type with `create_item`
- added the `OrderedCollectionItems` list type
- **BREAKING CHANGE** redefines the `OrderedCollectionItem` type with `create_item`
- added the `CollectionItems` list type
- **BREAKING CHANGE** redefines the `CollectionItem` type with `create_item`
- added `LinkItem` + `LinkItems`
  - represents fields that take a `Link | List<Link> | Iri` range
  - differentiated from `Item` by not accepting `Object` types
- deprecated the `IriItem` type
  - unhandled edge-case is addressed by `LinkItems`
- added `ObjectItem` + `ObjectItems`
  - represents fields that take a `Object | List<Object> | Iri` range
  - differentiated from `Item` by not accepting `Link` types
- deprecated the `Objects` type in favor of `ObjectItems`
  - unhandled edge-case is addressed by `ObjectItems`
- added new `create_list` helper macro
  - creates new list types with `Single` + `List` variants
  - useful for field types that can be a single object or list of items
- **BREAKING CHANGE** redefines a number of library list types using the `create_list` macro
  - some inconsistencies in previous definitions are now consistent
  - the new consistency breaks previous API contracts

## 0.4.2

- bump `base58ck` to `0.4.0`

## 0.4.1

- improve ergonomics for `KeyItems` + `MultikeyItems` functions
  - implement conversion traits on generics to improve usability of field access functions

## 0.4.0

- improve code organization by moving types into modules
  - moves `activity`, `actor`, `link`, and `object` types into modules
  - adds module-level documentation to public modules
  - re-exports module types from the top-level for convenience
- add `KeyItem` + `MutlikeyItem` types
  - enables representing lists of mixed object + IRI referencing key material
- **BREAKING CHANGE** changed `Actor` types to use item types in their key fields

## 0.3.1

- added `Alternatives` section to top-level documentation
  - mentioned influences from [go-ap/activitypub](https://github.com/go-ap/activitypub)
- added the `publicKey` field to all `Actor` types
  - marked as deprecated since Security Vocabulary 2.0 

## 0.3.0

- add `Key` type for `Security Vocabulary V1` + Mastodon compatibility
- small lint + ergonomics fixes
- **BREAKING CHANGE** simplified the `create_object` + `create_link` macros for better consistency
- **BREAKING CHANGE** renamed the `Source` types to `Content` for more general use
- added the `Links` type for single/list variants of `Link` types.
- added more MIME types
- improved helper macro support for field names
- added support for Data Integrity Proofs specified in [FEP-8b32](https://codeberg.org/fediverse/fep/src/branch/main/fep/8b32)
- various fixes and improvements to helper macros for generics
- minor fixes for types

## 0.2.0

- improved helper macro ergonomics
- additional helper macros to define `Actor` and `Actvity` types
  - added `Actor`-specific fields specified in the main [`ActivityPub` specification](https://www.w3.org/TR/activitypub)
- additional tests and documentation
  - more extensive top-level documentation about how to use `activitystreams-vocabulary`
- added fields for `Actor` types to support `FEP-512a` + `FEP-521b` implementations
  - added the `Multikey` types to handle [`CID 1.0 Mutlikey`](https://www.w3.org/TR/cid-1.0/#Multikey) encoding.
