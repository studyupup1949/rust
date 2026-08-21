
[![build](https://api.travis-ci.org/CanalTP/address-formatter-rs.svg)](https://travis-ci.org/CanalTP/address-formatter-rs)
[![doc](https://docs.rs/address-formatter-rs/badge.svg)](https://docs.rs/address-formatter-rs)

# address-formatter-rs
Universal international address formatter in Rust - data from https://github.com/OpenCageData/address-formatting

This crate is based on the amazing work of [OpenCage Data](https://github.com/OpenCageData/address-formatting/) who collected so many international formats of postal addresses.

The implementation is a port of the [PHP](https://github.com/predicthq/address-formatter-php/blob/master/src/Formatter.php), [perl](https://github.com/OpenCageData/perl-Geo-Address-Formatter/blob/master/lib/Geo/Address/Formatter.pm) and [js](https://github.com/fragaria/address-formatter/blob/master/src/index.js) implementation of the Opencage configurations.

This is used by [mimirsbrunn](https://github.com/canaltp/mimirsbrunn), a [geocoder](https://en.wikipedia.org/wiki/Geocoding), to have nicely formatted addreses and POI.

:warning: don't forget to initialize & update the git submodules, as they held the opencage configurations.

`git submodule update --init`

## Usage

Add `address-formatter` in the Cargo.toml.

```rust
#[macro_use] extern crate maplit; // just to ease the Place creation

use address_formatter::{Component, Formatter};
use Component::*;
let formatter = Formatter::default();

let data = hashmap!(
    City => "Toulouse",
    Country => "France",
    CountryCode => "FR",
    County => "Toulouse",
    HouseNumber => "17",
    Neighbourhood => "Lafourguette",
    Postcode => "31000",
    Road => "Rue du Médecin-Colonel Calbairac",
    State => "Midi-Pyrénées",
    Suburb => "Toulouse Ouest",
);

assert_eq!(
formatter.format(data).unwrap(),
r#"17 Rue du Médecin-Colonel Calbairac
31000 Toulouse
France
"#.to_owned()
)

```

## Developing

You need an up to date rust version:

`rustup update`

To run the tests (especially the one based on all the [opencage tests cases](./address-formatting/testcases)).

`cargo test`


## TODO

 * There here are still some failing tests on corner cases
 * Abbreviation handling
