# Changelog

## 0.3.0

### Breaking changes

- Removed empty tuples from enum variants (e.g. `Decoration::Roll()` is now just `Decoration::Roll`).
- Added `Note` enum for notes, rather than using `char`, and normalised octave.
- Allow multiple decorations on the same note.
- Added support for tied notes.
- Updated peg to 0.7

### Other changes

- Made fields of data types public, so they can actually be used once a file is parsed.
- Implemented `Copy`, `Clone` and `Eq` for various data types.
- Allow spaces before tune header field values.
