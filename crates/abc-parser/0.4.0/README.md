# rust-abc-2

ABC Parser written in rust using PEG.

## Usage

Add the package to your cargo dependencies.
```toml
[dependencies]
abc-parser = "0.4"
```

Then you can use the PEG generated rules through the abc module.
```rust
use abc_parser::abc;
use abc_parser::datatypes::*;

let parsed = abc::tune_book("X:1\nT:Example\nK:D\n").unwrap();
assert_eq!(
    parsed,
    TuneBook::new(
        vec![],
        None,
        vec![Tune::new(
            TuneHeader::new(vec![
                HeaderLine::Field(InfoField::new('X', "1".to_string()), None),
                HeaderLine::Field(InfoField::new('T', "Example".to_string()), None),
                HeaderLine::Field(InfoField::new('K', "D".to_string()), None)
            ]),
            None
        )]
    )
)
```

## Feature List

These are roughly taken in order from the abc standard.
  - [x] Comments
    - [x] Comment lines with preceding whitespace
  - [x] Info fields
    - [x] Inline fields
    - [x] Remarks
    - [ ] Macros
    - [ ] Redefining Symbols
    - [ ] Multiple Voices
    - [ ] Clefs and Transposition
  - [x] Field continuation
  - [x] Notes
    - [x] Pitch
    - [x] Accidentals
    - [x] Lengths
    - [x] Ties
    - [x] Broken Rhythm
      - [ ] Splitting grace notes e.g. A{g}<A
  - [x] Rests
  - [x] Beams
  - [x] Bars
  - [x] First and second repeats
    - [x] Shortened e.g. |1 and :|2
  - [x] Variant Endings
  - [x] Slurs
  - [x] Grace Notes
  - [x] Tuplets
  - [x] Decorations
  - [x] Symbol Lines
  - [x] Chords and Unisons
  - [x] Annotations
    - [ ] Chord Symbols
  - [x] Lyrics
  - [x] Multiple Voices
    - [ ] Voice grouping shorthand syntax
  - [ ] Text Strings
  - [x] Reserved characters
  - [x] Stylesheet Directives
  - [ ] Dialects
  - [x] Spacers

## History

The first version was an attempt to write the parser by hand, but using PEG is
much more maintainable. The older repo is here:
[https://gitlab.com/Askaholic/rust-abc](https://gitlab.com/Askaholic/rust-abc)
