# Releases

## 0.2.0

- improved helper macro ergonomics
- additional helper macros to define `Actor` and `Actvity` types
  - added `Actor`-specific fields specified in the main [`ActivityPub` specification](https://www.w3.org/TR/activitypub)
- additional tests and documentation
  - more extensive top-level documentation about how to use `activitystreams-vocabulary`
- added fields for `Actor` types to support `FEP-512a` + `FEP-521b` implementations
  - added the `Multikey` types to handle [`CID 1.0 Mutlikey`](https://www.w3.org/TR/cid-1.0/#Multikey) encoding.
