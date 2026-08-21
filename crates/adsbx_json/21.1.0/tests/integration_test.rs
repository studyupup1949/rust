use adsbx_json::v2::Response;
use std::str::FromStr;

#[test]
fn test_parse_adsbx_all() {
    let input = include_str!("v2-specimen-adsbx-all.json");
    let response = Response::from_str(input).unwrap();
    assert_eq!(12479, response.aircraft.len());
    // Count how many aircraft have nac_p == 0.
    let nac_p_zero = response
        .aircraft
        .iter()
        .filter(|a| a.nac_p == Some(0) && a.lat.is_some() && a.lon.is_some())
        .count();
    assert_eq!(221, nac_p_zero);
}

#[test]
fn test_parse_adsbx_nearby() {
    let input = include_str!("v2-specimen-adsbx-nearby.json");
    let response = Response::from_str(input).unwrap();
    assert_eq!(305, response.aircraft.len());
}

#[test]
fn test_parse_airplaneslive() {
    let input = include_str!("v2-specimen-airplaneslive.json");
    let response = Response::from_str(input).unwrap();
    assert_eq!(76, response.aircraft.len());
    // Verify airplanes.live-specific fields are present
    let with_description = response
        .aircraft
        .iter()
        .filter(|a| a.description.is_some())
        .count();
    assert!(with_description > 0, "Expected some aircraft with description field");
}
