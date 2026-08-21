//! This crate defines data types according to the
//! [asset administration shell specifications](https://industrialdigitaltwin.org/en/content-hub/aasspecifications)
//!
//! To support multiple specs and multiple versions of each, this crate it split in
//! multiple modules for each part as well as version.
//! Because each spec is versioned on their own, the modules are ordered `specs/version` instead of
//! `version/specs`, i.e. `aas::part1::v3.1`.

/// Part1: Metamodel
pub mod part_1;

/// Utility functions like validating text to specific formats and deserializers to specific needs,
/// like text with defined constraints.
pub mod utilities;
