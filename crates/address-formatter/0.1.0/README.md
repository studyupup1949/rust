
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
use address_formatter::{Address, Component, Formatter};

let formatter = Formatter::default();

let mut addr = Address::default();
addr[Component::City] = Some("Toulouse".to_owned());
addr[Component::Country] = Some("France".to_owned());
addr[Component::CountryCode] = Some("FR".to_owned());
addr[Component::County] = Some("Toulouse".to_owned());
addr[Component::HouseNumber] = Some("17".to_owned());
addr[Component::Neighbourhood] = Some("Lafourguette".to_owned());
addr[Component::Postcode] = Some("31000".to_owned());
addr[Component::Road] = Some("Rue du Médecin-Colonel Calbairac".to_owned());
addr[Component::State] = Some("Midi-Pyrénées".to_owned());
addr[Component::Suburb] = Some("Toulouse Ouest".to_owned());

assert_eq!(
        formatter.format(addr).unwrap(),
r#"17 Rue du Médecin-Colonel Calbairac
31000 Toulouse
France
"#
        .to_owned()
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
