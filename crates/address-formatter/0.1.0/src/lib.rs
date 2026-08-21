#![deny(missing_docs)]

//! Universal international address formatter in Rust - data from https://github.com/OpenCageData/address-formatting
//!
//! This crate is based on the amazing work of [OpenCage Data](https://github.com/OpenCageData/address-formatting/)
//! who collected so many international formats of postal addresses.
//!
//! The easier way to use this crate is to create an [`Address`](struct.Address.html) and [`format`](struct.Formatter.html#method.format) it.
//! The [`Formatter`](struct.Formatter.html) will try to autodetect the country of the [`Address`](struct.Address.html)
//! (this detection can be overriden with some [`Configuration`](struct.Configuration.html))
//! and format the postal address using the opencage rules for this country.
//!  
//! ```
//! # #[macro_use] extern crate maplit;
//! # fn main() {
//!    use address_formatter::Component::*;
//!    let formatter = address_formatter::Formatter::default();
//!
//!    // create an Address from a HashMap.
//!    // We could also have created an Address by settings all its fields
//!    let addr: address_formatter::Address = hashmap!(
//!        City => "Toulouse",
//!        Country => "France",
//!        CountryCode => "FR",
//!        County => "Toulouse",
//!        HouseNumber => "17",
//!        Neighbourhood => "Lafourguette",
//!        Postcode => "31000",
//!        Road => "Rue du Médecin-Colonel Calbairac",
//!        State => "Midi-Pyrénées",
//!        Suburb => "Toulouse Ouest",
//!    ).into();
//!
//!    assert_eq!(
//!        formatter.format(addr).unwrap(),
//!        r#"17 Rue du Médecin-Colonel Calbairac
//!31000 Toulouse
//!France
//!"#
//!        .to_owned()
//!    )
//! # }
//! ```
//!
//! If your data are less stuctured, you can use the [`AddressBuilder`](struct.AddressBuilder.html) to build an [`Address`](struct.Address.html)
//!
//!
//! ```
//! # fn main() {
//!    let formatter = address_formatter::Formatter::default();
//!    let addr_builder = address_formatter::AddressBuilder::default();
//!    let data = [
//!        ("building", "Mairie (bureaux administratifs)"),
//!        ("city", "Papeete"),
//!        (
//!            "country",
//!            "Polynésie française, Îles du Vent (eaux territoriales)",
//!        ),
//!        ("country_code", "fr"),
//!        ("county", "Îles du Vent"),
//!        ("postcode", "98714"),
//!        ("road", "Rue des Remparts"),
//!        ("state", "French Polynesia"),
//!    ];
//!
//!    let addr =
//!        addr_builder.build_address(data.into_iter().map(|(k, v)| (k.clone(), v.to_string())));
//!
//!    assert_eq!(
//!        formatter.format(addr).unwrap(),
//!        r#"Mairie (bureaux administratifs)
//!Rue des Remparts
//!98714 Papeete
//!Polynésie française
//!"#
//!        .to_owned()
//!    )
//! # }
//! ```

pub(crate) mod address;
pub(crate) mod formatter;
pub(crate) mod handlebar_helper;
pub(crate) mod read_configuration;

pub use address::{Address, Component};
pub use formatter::{AddressBuilder, Configuration, Formatter};
