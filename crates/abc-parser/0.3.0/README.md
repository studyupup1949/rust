# rust-abc-2

ABC Parser written in rust using PEG.

## Usage

Add the package to your cargo dependencies.
```toml
[dependencies]
abc-parser = "0.2"
```

Then you can use the PEG generated rules through the abc module.
```rust
extern crate abc_parser;

use abc_parser::datatypes::*;
use abc_parser::abc;

let parsed = abc::tune_book("X:1\nT:Example\nK:D\n").unwrap();
assert_eq!(
    parsed,
    TuneBook::new(None, vec![
        Tune::new(
            TuneHeader::new(vec![
                InfoField::new('X', "1".to_string()),
                InfoField::new('T', "Example".to_string()),
                InfoField::new('K', "D".to_string())
            ]),
            None
        )
    ])
)
```

There is a good chance that the `dev` branch may have more recently added
features which arent complete but won't (probably) break the existing
functionality. You can also tell cargo to install directly from the `dev` branch
like this:
```toml
[dependencies]
abc-parser = { git = "https://gitlab.com/Askaholic/rust-abc-2", branch = "dev" }
```

## Feature List

These are roughly taken in order from the abc standard.
  - [x] Info fields
  - [ ] Field continuation
  - [ ] Notes
    - [x] Pitch
    - [x] Accidentals
    - [x] Lengths
    - [x] Ties
    - [ ] Broken Rhythm
  - [x] Rests
  - [ ] Clefs and Transposition
  - [x] Beams
  - [x] Bars
  - [ ] Variant Endings
  - [ ] Slurs
  - [x] Grace Notes
  - [x] Tuplets
  - [x] Decorations
  - [ ] Symbol Lines
  - [ ] Redefining Symbols
  - [x] Chords and Unisons
  - [ ] Chord Symbols
  - [ ] Annotations
  - [ ] Lyrics
  - [ ] Multiple Voices
  - [ ] Text Strings
  - [ ] Macros
  - [ ] Stylesheet Directives
  - [ ] Dialects

## History

The first version was an attempt to write the parser by hand, but using PEG is
much more maintainable. The older repo is here:
[https://gitlab.com/Askaholic/rust-abc](https://gitlab.com/Askaholic/rust-abc)
