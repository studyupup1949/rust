use adsbx_json::v2::Response;
use std::str::FromStr;

#[test]
fn test_parse_lots_of_aircraft() {
    let input = include_str!("v2-specimen.json");
    let response = Response::from_str(input).unwrap();
    assert_eq!(8668, response.aircraft.len());
}
