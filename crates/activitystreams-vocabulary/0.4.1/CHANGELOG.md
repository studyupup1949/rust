# Releases

## Unreleased

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
