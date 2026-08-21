# Changelog

## 0.4.0

### Breaking changes 
- Lengths are now expressed as a newtype wrapper for f32 `Option<Length>`
- Rests now use the same `Length` type as notes
- Tune bodies now consist of `TuneLine`s rather than `MusicLine`s in order to 
  support symbol and lyric lines
- `Tuplet`s no longer are parsed to include the following vector of notes
- `Decoration`s are now stand alone symbols and are no longer attached to `Note`s
- `GraceNote`s can now contain `BrokenRhythm`s and `Space`s but no longer can 
contain `Decoration`s
- `VisualBreak` is now `Space` and contains the original whitespace string
- `FileHeader.info` is now `FileHeader.lines` and can contain comment lines
- `HeaderLine::Field` can now contain comments

### Other changes
- Lengths are converted back to ABC more accurately
- Beams are now captured as `MusicSymbol::Beam`
- Added `ToABC::write_abc` and `ToABC::write_mut_abc` to reduce the number of 
allocations needed when converting tunes back to ABC
- Fixed a bug where upper octaves were rendered with an extra '
- Added missing `ToABC` implementation for Grace Notes, Chords, and Tuplets
- Updated peg to 0.8

#### New supported syntax
- Added field continuation
- Added reserved characters
- Added spacers
- Added lyric lines
- Added symbol lines
- Added slurs
- Added broken rhythm
- Added first and second repeats and general variant endings
- Added annotations
- Added inline fields
- Added dotted barlines
- Added comments and stylesheet directives

## 0.3.0

### Breaking changes

- Removed empty tuples from enum variants (e.g. `Decoration::Roll()` is now just `Decoration::Roll`).
- Added `Note` enum for notes, rather than using `char`, and normalize octave.
- Allow multiple decorations on the same note.
- Added support for tied notes.
- Updated peg to 0.7

### Other changes

- Made fields of data types public, so they can actually be used once a file is parsed.
- Implemented `Copy`, `Clone` and `Eq` for various data types.
- Allow spaces before tune header field values.
