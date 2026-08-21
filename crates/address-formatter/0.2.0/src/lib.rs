#![deny(missing_docs)]

//! Universal international place formatter in Rust
//!
//! This crate is based on the amazing work of [OpenCage Data](https://github.com/OpenCageData/place-formatting/)
//! who collected so many international formats of postal placees.
//!
//! The easiest way to use this crate is to create a [`Place`](struct.Place.html) and [`format`](struct.Formatter.html#method.format) it.
//! The [`Formatter`](struct.Formatter.html) will try to autodetect the country of the [`Place`](struct.Place.html)
//! (this detection can be overriden with some [`Configuration`](struct.Configuration.html))
//! and format the postal place using the opencage rules for this country.
//!  
//! ```
//! # #[macro_use] extern crate maplit;
//! # fn main() {
//!    use address_formatter::Component::*;
//!
//!    // create a Place from a HashMap.
//!    // We could also have created a Place by settings all its fields
//!    let addr: address_formatter::Place = hashmap!(
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
//!        address_formatter::FORMATTER.format(addr).unwrap(),
//!        r#"17 Rue du Médecin-Colonel Calbairac
//!31000 Toulouse
//!France
//!"#
//!        .to_owned()
//!    )
//! # }
//! ```
//!
//! If your data are less stuctured, you can use the [`PlaceBuilder`](struct.PlaceBuilder.html) to build a [`Place`](struct.Place.html)
//!
//!
//! ```
//! # fn main() {
//!    // use can either use the provider singleton address_formatter::FORMATTER or build your own
//!    let formatter = address_formatter::Formatter::default();
//!    let addr_builder = address_formatter::PlaceBuilder::default();
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
//!        addr_builder.build_place(data.into_iter().map(|(k, v)| (k.clone(), v.to_string())));
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

pub(crate) mod formatter;
pub(crate) mod handlebar_helper;
pub(crate) mod place;
pub(crate) mod read_configuration;

pub use formatter::{Configuration, Formatter, PlaceBuilder};
pub use place::{Component, Place};

lazy_static::lazy_static! {
    /// Singleton to ease use of the [`Formatter`](struct.Formatter.html)
    ///
    /// ```
    /// # #[macro_use] extern crate maplit;
    /// # fn main() {
    ///    assert_eq!(
    ///        address_formatter::FORMATTER
    ///            .format(hashmap!(
    ///                address_formatter::Component::City => "Toulouse",
    ///                address_formatter::Component::Country => "France",
    ///                address_formatter::Component::CountryCode => "FR",
    ///                address_formatter::Component::Road => "Rue du Médecin-Colonel Calbairac",
    ///            ))
    ///            .unwrap(),
    ///        r#"Rue du Médecin-Colonel Calbairac
    ///Toulouse
    ///France
    ///"#.to_owned()
    ///    );
    /// # }
    /// ```
    pub static ref FORMATTER: Formatter = Formatter::default();
}
