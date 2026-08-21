#[macro_use]
extern crate maplit;
use address_formatter::{Component, Formatter, Place};

#[test]
pub fn basic_test() {
    let formatter = Formatter::default();

    let mut addr = Place::default();
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
        formatter.short_addr_format(addr).unwrap(),
        r#"17 Rue du Médecin-Colonel Calbairac"#.to_owned()
    )
}

// Same test as above, except we use another way to initialize a place
// (from a hashmap)
#[test]
pub fn easier_init_test() {
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
        formatter.short_addr_format(data).unwrap(),
        r#"17 Rue du Médecin-Colonel Calbairac"#.to_owned()
    )
}

#[test]
pub fn empty_address() {
    let formatter = Formatter::default();
    let addr = Place::default();
    assert_eq!(formatter.short_addr_format(addr).unwrap(), "".to_owned())
}

#[test]
pub fn addr_fmt_for_country_without_housenumber() {
    use Component::*;
    // Antarctica has no housenumber in their format (strange isn't it :D)
    // we should not be able to have a shot_addr_format
    let formatter = Formatter::default();
    let data = hashmap!(
        Attention => "Oil Separator Building",
        City => "McMurdo Station",
        Country => "Antarctica",
        CountryCode => "aq",
        HouseNumber => "72",
        Road => "McMurdo Roads",
    );
    assert!(formatter.short_addr_format(data).is_err())
}
