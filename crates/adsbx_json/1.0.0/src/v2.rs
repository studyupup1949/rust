//! Parses [version 2](https://www.adsbexchange.com/version-2-api-wip/) of the
//! ADS-B Exchange API JSON format.
//!
//! ```
//! use adsbx_json::v2::Response;
//! use std::str::FromStr;
//! 
//! # let json_str = include_str!("../tests/v2-specimen-01.json");
//! let response = Response::from_str(&json_str).unwrap();
//! println!("Got {} aircraft", response.aircraft.len());
//! println!("1st aircraft:\n{:#?}", response.aircraft[0]);
//! ```

use bitflags::bitflags;
use bitflags_serde_shim::impl_serde_for_bitflags;
use chrono::serde::ts_milliseconds;
use chrono::{prelude::*};
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use serde_with::{serde_as, DurationMilliSeconds, DurationSeconds};
use std::{error::Error, fmt, num::ParseIntError, str::FromStr};

/// Represents a parsing error.
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

/// Describes the source of a message.
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
/// Flags an aircraft may have in the ADS-B Exchange database.
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

/// Describes the status of an aircraft, either airborne or on the ground.
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum SilType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "perhour")]
    PerHour,
    #[serde(rename = "persample")]
    PerSample,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Emergency {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "general")]
    General,
    #[serde(rename = "lifeguard")]
    Lifeguard,
    #[serde(rename = "minfuel")]
    MinFuel,
    #[serde(rename = "nordo")]
    Nordo,
    #[serde(rename = "unlawful")]
    Unlawful,
    #[serde(rename = "downed")]
    Downed,
}

fn optional_bool_from_int<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<u8>::deserialize(deserializer)? {
        Some(v) => match v {
            0 => Ok(Some(false)),
            1 => Ok(Some(true)),
            other => Err(de::Error::invalid_value(
                Unexpected::Unsigned(other as u64),
                &"zero or one",
            )),
        },
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum NavMode {
    #[serde(rename = "althold")]
    AltitudeHold,
    #[serde(rename = "approach")]
    Approach,
    #[serde(rename = "autopilot")]
    Autopilot,
    #[serde(rename = "lnav")]
    LNav,
    #[serde(rename = "tcas")]
    Tcas,
    #[serde(rename = "vnav")]
    VNav,
}

/// An aircraft. (Might not actually be an aircraft—could be a ground vehicle,
/// or a test beacon.)
#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Aircraft {
    /// The version of ADS-B the aircraft is using.
    #[serde(rename = "version")]
    pub adsb_version: Option<u8>,

    // The aircraft type. E.g. "C172".
    #[serde(rename = "t")]
    pub aircraft_type: Option<String>,

    #[serde(rename = "baro_rate")]
    pub barometric_vertical_rate: Option<i16>,

    #[serde(rename = "alt_baro")]
    pub barometric_altitude: Option<AltitudeOrGround>,

    pub calc_track: Option<u16>,

    #[serde(rename = "flight")]
    pub call_sign: Option<String>,

    #[serde(rename = "dbFlags", default)]
    pub database_flags: DatabaseFlags,
    pub emergency: Option<Emergency>,

    #[serde(rename = "category")]
    pub emitter_category: Option<String>,

    #[serde(rename = "alt_geom")]
    pub geometric_altitude: Option<i32>,

    #[serde(rename = "gva")]
    pub geometric_vertical_accuracy: Option<u8>,

    #[serde(rename = "geom_rate")]
    pub geometric_vertical_rate: Option<i16>,

    #[serde(rename = "gs")]
    pub ground_speed_knots: Option<f32>,

    pub hex: String,

    #[serde(rename = "ias")]
    pub indicated_air_speed_knots: Option<f32>,

    #[serde(rename = "alert", default, deserialize_with = "optional_bool_from_int")]
    pub is_alert: Option<bool>,

    pub lat: Option<f32>,

    pub lon: Option<f32>,

    #[serde(rename = "mach")]
    pub mach: Option<f32>,

    #[serde(rename = "mag_heading")]
    pub magnetic_heading: Option<f32>,

    #[serde(rename = "type")]
    pub message_type: MessageType,

    #[serde(rename = "mlat")]
    pub mlat_fields: Option<Vec<String>>,

    pub nac_p: Option<u8>,

    pub nac_v: Option<u8>,

    pub nav_altitude_fms: Option<u16>,

    pub nav_altitude_mcp: Option<i32>,

    pub nav_heading: Option<f32>,

    pub nav_modes: Option<Vec<NavMode>>,

    pub nav_qnh: Option<f32>,

    pub nic: Option<u8>,

    pub nic_baro: Option<u8>,

    #[serde(rename = "messages")]
    pub num_messages: i32,

    #[serde(rename = "oat")]
    pub outside_air_temperature: Option<i32>,

    #[serde(rename = "rc")]
    pub radius_of_containment_meters: Option<u32>,

    #[serde(rename = "r")]
    pub registration: Option<String>,

    pub roll: Option<f32>,

    pub rr_lat: Option<f32>,

    pub rr_lon: Option<f32>,

    pub rssi: f32,

    #[serde_as(as = "DurationSeconds<f64>")]
    pub seen: std::time::Duration,

    #[serde_as(as = "Option<DurationSeconds<f64>>")]
    #[serde(default)]
    pub seen_pos: Option<std::time::Duration>,

    pub sil: Option<u8>,

    pub sil_type: Option<SilType>,

    #[serde(default, deserialize_with = "optional_bool_from_int")]
    pub spi: Option<bool>,

    pub squawk: Option<String>,

    #[serde(rename = "sda")]
    pub system_design_assurance: Option<u8>,

    #[serde(rename = "tisb")]
    pub tisb_fields: Option<Vec<String>>,

    #[serde(rename = "tat")]
    pub total_air_temperature: Option<i32>,

    pub track: Option<f32>,

    pub track_rate: Option<f32>,

    #[serde(rename = "tas")]
    pub true_air_speed_knots: Option<f32>,

    pub true_heading: Option<f32>,

    #[serde(rename = "wd")]
    pub wind_direction: Option<u16>,

    #[serde(rename = "ws")]
    pub wind_speed: Option<u16>,
}

/// The ADS-B Exchange API response. This has most of the info you want :)
///
/// See <https://www.adsbexchange.com/version-2-api-wip/>
///
/// Holds the aircraft returned, and some metadata about the response.
///
/// ```
/// use adsbx_json::v2::Response;
/// use std::str::FromStr;
///
/// # let json_str = include_str!("../tests/v2-specimen-01.json");
/// let response = Response::from_str(&json_str).unwrap();
/// println!("Got {} aircraft", response.aircraft.len());
/// println!("1st aircraft:\n{:#?}", response.aircraft[0]);
/// ```
#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Response {
    /// The time this response was generated.
    #[serde(with = "ts_milliseconds")]
    pub now: DateTime<Utc>,

    /// The time this response was cached.
    #[serde(rename = "ctime")]
    #[serde(with = "ts_milliseconds")]
    pub cache_time: DateTime<Utc>,

    /// Error message, or if there isn't an error, "No error".
    #[serde(rename = "msg")]
    pub message: String,

    /// Server processing time the request required.
    #[serde(rename = "ptime")]
    #[serde_as(as = "DurationMilliSeconds<f64>")]
    pub processing_time: std::time::Duration,

    /// Total aircraft returned.
    #[serde(rename = "total")]
    pub num_aircraft: u64,

    /// Vector of all known aircraft.
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
          {"now": 1614109133600,
           "ctime": 1614109134570,
           "msg": "No error",
           "ptime": 61,
           "total": 1,
           "ac": [{
                   "alert": 1,
                   "alt_baro": "ground",
                   "alt_geom": 123,
                   "baro_rate": 1350,
                   "calc_track": 43,
                   "category": "A0",
                   "dbFlags": 0,
                   "emergency": "none",
                   "flight": "N1234",
                   "geom_rate": 1000,
                   "gs": 50,
                   "gva": 1,
                   "hex":"000000",
                   "ias": 45.1,
                   "lat": 1.1,
                   "lon": 1.2,
                   "mach": 0.88,
                   "mag_heading": 12.3,
                   "messages": 5,
                   "nac_p": 4,
                   "nac_v": 4,
                   "nav_altitude_fms": 16000,
                   "nav_altitude_mcp": 12000,
                   "nav_heading": 99.9,
                   "nav_modes": ["tcas", "approach"],
                   "nav_qnh": 1014,
                   "nic": 11,
                   "nic_baro": 1,
                   "oat": -15,
                   "r": "N1234",
                   "rc": 18520,
                   "roll": -3.1,
                   "rssi": -4.5,
                   "sda": 0,
                   "seen": 54,
                   "seen_pos": 32,
                   "sil": 0,
                   "sil_type": "unknown",
                   "spi": 0,
                   "squawk": "1234",
                   "t": "C172",
                   "tas": 12.3,
                   "tat": -10,
                   "track": 24.2,
                   "track_rate": 10.1,
                   "true_heading": 35.3,
                   "type": "adsb_icao",
                   "version": 2,
                   "wd": 110,
                   "ws": 45
                }]}
        "#;
        let response = Response::from_str(input).unwrap();
        assert_eq!(
            response,
            Response {
                now: Utc.ymd(2021, 2, 23).and_hms_milli(19, 38, 53, 600),
                cache_time: Utc.ymd(2021, 2, 23).and_hms_milli(19, 38, 54, 570),
                message: "No error".to_string(),
                processing_time: std::time::Duration::new(0, 61000000),
                num_aircraft: 1,
                aircraft: vec![Aircraft {
                    adsb_version: Some(2),
                    aircraft_type: Some("C172".to_string()),
                    barometric_vertical_rate: Some(1350),
                    barometric_altitude: Some(AltitudeOrGround::OnGround),
                    calc_track: Some(43),
                    call_sign: Some("N1234".to_string()),
                    database_flags: DatabaseFlags::empty(),
                    emergency: Some(Emergency::None),
                    emitter_category: Some("A0".to_string()),
                    geometric_altitude: Some(123),
                    geometric_vertical_accuracy: Some(1),
                    geometric_vertical_rate: Some(1000),
                    ground_speed_knots: Some(50.0),
                    hex: "000000".to_string(),
                    indicated_air_speed_knots: Some(45.1),
                    is_alert: Some(true),
                    lat: Some(1.1),
                    lon: Some(1.2),
                    mach: Some(0.88),
                    magnetic_heading: Some(12.3),
                    message_type: MessageType::AdsBIcao,
                    mlat_fields: None,
                    nac_p: Some(4),
                    nac_v: Some(4),
                    nav_altitude_fms: Some(16000),
                    nav_altitude_mcp: Some(12000),
                    nav_heading: Some(99.9),
                    nav_modes: Some([NavMode::Tcas, NavMode::Approach].to_vec()),
                    nav_qnh: Some(1014.0),
                    nic: Some(11),
                    nic_baro: Some(1),
                    num_messages: 5,
                    outside_air_temperature: Some(-15),
                    radius_of_containment_meters: Some(18520),
                    registration: Some("N1234".to_string()),
                    roll: Some(-3.1),
                    rr_lat: None,
                    rr_lon: None,
                    rssi: -4.5,
                    seen: std::time::Duration::new(54, 0),
                    seen_pos: Some(std::time::Duration::new(32, 0)),
                    sil: Some(0),
                    sil_type: Some(SilType::Unknown),
                    spi: Some(false),
                    squawk: Some("1234".to_string()),
                    system_design_assurance: Some(0),
                    tisb_fields: None,
                    total_air_temperature: Some(-10),
                    track: Some(24.2),
                    track_rate: Some(10.1),
                    true_air_speed_knots: Some(12.3),
                    true_heading: Some(35.3),
                    wind_direction: Some(110),
                    wind_speed: Some(45),
                }]
            }
        );
    }
}
