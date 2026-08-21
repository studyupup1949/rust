use adsbx_json::v2::Response;
use std::str::FromStr;

#[test]
fn test_parse_all_aircraft() {
    let input = include_str!("v2-specimen-all.json");
    let response = Response::from_str(input).unwrap();
    assert_eq!(9085, response.aircraft.len());
}

#[test]
fn test_parse_nearby_aircraft() {
    let input = include_str!("v2-specimen-nearby.json");
    let response = Response::from_str(input).unwrap();
    assert_eq!(370, response.aircraft.len());
}
