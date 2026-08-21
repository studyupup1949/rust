//! This crate defines data types according to the
//! [asset administration shell specifications](https://industrialdigitaltwin.org/en/content-hub/aasspecifications)
//!
//! To support multiple specs and multiple versions of each, this crate it split in
//! multiple modules for each part as well as version.
//! Because each spec is versioned on their own, the modules are ordered `specs/version` instead of
//! `version/specs`, i.e. `aas::part1::v3.1`.

/// Part1: Metamodel
pub mod part1;

/// Part 2: Http Endpoints
/// Can be used with feature = "part2"
#[cfg(feature = "part2")]
pub mod part2;

/// Utility functions like validating text to specific formats and deserializers to specific needs,
/// like text with defined constraints.
pub mod utilities;

// Aufbau:
// if feature xml: pub use xml::Struct
// if feature json: pub use json::Struct
// if both: pub use xml::Struct as StructXML,
//          pub use json::Struct as StructJSON
//          pub Struct (without Des, Ser)
// dabei wird das macro generate.. genutzt.
// in den jeweiligen modulen (xml, json) wird dann impl Ser, Der
// implementiert. Dabei kann innerhalb der implementierung das geeignete
// struct (z.B. für flatten) benutzt werden.

// das wird erst umgesetzt, nachdem die erste Version von XML steht.
