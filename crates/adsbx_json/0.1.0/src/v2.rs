use bitflags::bitflags;
use bitflags_serde_shim::impl_serde_for_bitflags;
use serde::{Deserialize, Deserializer, Serialize};
use std::{error::Error, fmt, num::ParseIntError, str::FromStr};

/// https://www.adsbexchange.com/version-2-api-wip/

#[derive(Debug)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseError {}

impl From<serde_json::Error> for ParseError
where
    serde_json::Error: std::fmt::Debug,
{
    fn from(error: serde_json::Error) -> Self {
        ParseError(format!("{:?}", error))
    }
}

impl From<ParseIntError> for ParseError
where
    serde_json::Error: std::fmt::Debug,
{
    fn from(error: ParseIntError) -> Self {
        ParseError(format!("{:?}", error))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageType {
    #[serde(rename = "adsb_icao")]
    AdsBIcao,
    #[serde(rename = "adsb_icao_nt")]
    AdsBIcaoNonTransponder,
    #[serde(rename = "adsb_other")]
    AdsBOther,
    #[serde(rename = "adsc")]
    AdsC,
    #[serde(rename = "adsr_icao")]
    AdsRIcao,
    #[serde(rename = "adsr_other")]
    AdsROther,
    #[serde(rename = "mode_s")]
    ModeS,
    #[serde(rename = "mlat")]
    Multilateration,
    #[serde(rename = "other")]
    Other,
    #[serde(rename = "tisb_icao")]
    TisBIcao,
    #[serde(rename = "tisb_other")]
    TisBOther,
    #[serde(rename = "tisb_trackfile")]
    TisBTrackfile,
    #[serde(rename = "unknown")]
    Unknown,
}

bitflags! {
    pub struct DatabaseFlags: u32 {
        const MILITARY = 0b00000001;
        const INTERESTING = 0b00000010;
    }
}

impl_serde_for_bitflags!(DatabaseFlags);

impl std::default::Default for DatabaseFlags {
    fn default() -> Self {
        DatabaseFlags { bits: 0 }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum AltitudeOrGround {
    Altitude(i32),
    #[serde(deserialize_with = "ground_altitude")]
    OnGround,
}

// Wow, I really struggled with this before finding
// https://github.com/serde-rs/serde/issues/1158#issuecomment-365362959

fn ground_altitude<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    enum Helper {
        #[serde(rename = "ground")]
        Variant,
    }
    Helper::deserialize(deserializer)?;
    Ok(())
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Aircraft {
    pub hex: String,
    #[serde(rename = "type")]
    pub message_type: MessageType,
    #[serde(rename = "flight")]
    pub call_sign: Option<String>,
    #[serde(rename = "r")]
    pub registration: Option<String>,
    #[serde(rename = "t")]
    pub aircraft_type: Option<String>,
    #[serde(rename = "dbFlags", default)]
    pub database_flags: DatabaseFlags,
    #[serde(rename = "alt_baro")]
    pub barometric_altitude: Option<AltitudeOrGround>,
    #[serde(rename = "alt_geom")]
    pub geometric_altitude: Option<i32>,
    #[serde(rename = "gs")]
    pub ground_speed_knots: Option<f32>,
    #[serde(rename = "ias")]
    pub indicated_air_speed_knots: Option<f32>,
    #[serde(rename = "tas")]
    pub true_air_speed_knots: Option<f32>,
    #[serde(rename = "mach")]
    pub mach: Option<f32>,
    pub track: Option<f32>,
    #[serde(rename = "mag_heading")]
    pub magnetic_heading: Option<f32>,
    pub squawk: Option<String>,
    pub lat: Option<f32>,
    pub lon: Option<f32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Response {
    #[serde(rename = "ac")]
    pub aircraft: Vec<Aircraft>,
}

impl FromStr for Response {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let response: Response = serde_json::from_str(input)?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_parse_aircraft() {
        let input = r#"
          {"ac": [{"hex":"000000",
                   "r": "N1234",
                   "t": "C172",
                   "type": "adsb_icao",
                   "flight": "N1234",
                   "dbFlags": 0,
                   "alt_baro": "ground",
                   "alt_geom": 123,
                   "gs": 50,
                   "ias": 45.1,
                   "tas": 12.3,
                   "mach": 0.88,
                   "track": 24.2,
                   "mag_heading": 12.3,
                   "squawk": "1234",
                   "lat": 1.1,
                   "lon": 1.2}]}
        "#;
        let response = Response::from_str(input).unwrap();
        assert_eq!(
            response,
            Response {
                aircraft: vec![Aircraft {
                    hex: "000000".to_string(),
                    message_type: MessageType::AdsBIcao,
                    call_sign: Some("N1234".to_string()),
                    registration: Some("N1234".to_string()),
                    aircraft_type: Some("C172".to_string()),
                    database_flags: DatabaseFlags::empty(),
                    barometric_altitude: Some(AltitudeOrGround::OnGround),
                    geometric_altitude: Some(123),
                    ground_speed_knots: Some(50.0),
                    indicated_air_speed_knots: Some(45.1),
                    true_air_speed_knots: Some(12.3),
                    mach: Some(0.88),
                    track: Some(24.2),
                    magnetic_heading: Some(12.3),
                    squawk: Some("1234".to_string()),
                    lat: Some(1.1),
                    lon: Some(1.2),
                }]
            }
        );
    }
}
