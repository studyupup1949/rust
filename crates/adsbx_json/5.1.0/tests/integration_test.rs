use adsbx_json::v2::Response;
use std::str::FromStr;

#[test]
fn test_parse_lots_of_aircraft() {
    {
        let input = include_str!("v2-specimen-all.json");
        let response = Response::from_str(input).unwrap();
        assert_eq!(8286, response.aircraft.len());
    }
    {
        let input = include_str!("v2-specimen-nearby.json");
        let response = Response::from_str(input).unwrap();
        assert_eq!(386, response.aircraft.len());
    }
}
