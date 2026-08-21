//! Parses [version 2](https://www.adsbexchange.com/version-2-api-wip/) of the
//! ADS-B Exchange API JSON format.
//!
//! ```
//! use adsbx_json::v2::Response;
//! use std::str::FromStr;
//!
//! # let json_str = include_str!("../tests/v2-specimen-nearby.json");
//! let response = Response::from_str(&json_str).unwrap();
//! println!("Got {} aircraft", response.aircraft.len());
//! let ac = &response.aircraft[0];
//! println!("ICAO: {}", ac.hex);
//! if let Some(reg) = &ac.registration {
//!     println!("Registration: {}", reg);
//! }
//! if let (Some(lat), Some(lon)) = (ac.lat, ac.lon) {
//!     println!("Aircraft is at {}, {}", lat, lon);
//! }
//! ```

use chrono::prelude::*;
use chrono::serde::ts_milliseconds;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use serde_with::{serde_as, DurationMilliSeconds, DurationSeconds, TimestampSecondsWithFrac};
use std::str::FromStr;

use crate::ParseError;

/// Describes the source of a message.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

/// Flags an aircraft may have in the ADS-B Exchange database.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DatabaseFlags(u32);

impl DatabaseFlags {
    pub const MILITARY: u32 = 0b1;
    pub const INTERESTING: u32 = 0b10;
    pub const PIA: u32 = 0b100;
    pub const LADD: u32 = 0b1000;

    /// The aircraft is military.
    pub fn is_military(&self) -> bool {
        self.0 & Self::MILITARY != 0
    }

    /// The aircraft is "interesting" for some reason. It could be federal law
    /// enforcement, or NASA, or something else.
    pub fn is_interesting(&self) -> bool {
        self.0 & Self::INTERESTING != 0
    }

    /// The aircraft is using a private ICAO address. See
    /// <https://www.faa.gov/nextgen/equipadsb/privacy/>
    pub fn is_pia(&self) -> bool {
        self.0 & Self::PIA != 0
    }

    /// The aircraft has requested that it not be included in the FAA's data
    /// feeds for flight tracking. See <https://ladd.faa.gov/>
    pub fn is_ladd(&self) -> bool {
        self.0 & Self::LADD != 0
    }
}

impl std::default::Default for DatabaseFlags {
    fn default() -> Self {
        DatabaseFlags(0)
    }
}

/// Describes the status of an aircraft, either airborne or on the ground.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum SilType {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "perhour")]
    PerHour,
    #[serde(rename = "persample")]
    PerSample,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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
    #[serde(rename = "reserved")]
    Reserved,
}

// Deserializes 1/0 into true/false, or None.

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

// Deserializes an array of aircraft, but if the array is null then deserializes
// it as an empty vector.

fn empty_aircraft_vec<'de, D>(deserializer: D) -> Result<Vec<Aircraft>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Vec<Aircraft>>::deserialize(deserializer)? {
        Some(ac) => Ok(ac),
        None => Ok(vec![]),
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AgedPosition {
    #[serde_as(as = "DurationSeconds<f64>")]
    pub seen_pos: std::time::Duration,
    pub lat: f64,
    pub lon: f64,
    pub nic: u8,
    pub rc: u32,
}

// "ARA": "1100001",
// "MTE": "0",
// "RAC": "0000",
// "RAT": "1",
// "TTI": "01",
// "advisory": "Clear of Conflict",
// "advisory_complement": "",
// "bytes": "E2C20026B7FF0C",
// "threat_id_hex": "adffc3",
// "unix_timestamp": 1617657530.42,
// "utc": "2021-04-05 21:18:50.4"

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AcasRa {
    #[serde(rename = "ARA")]
    ara: String,
    #[serde(rename = "MTE")]
    mte: String,
    #[serde(rename = "RAC")]
    rac: String,
    #[serde(rename = "RAT")]
    rat: String,
    #[serde(rename = "TTI")]
    tti: String,
    advisory: String,
    advisory_complement: String,
    bytes: String,
    threat_id_hex: Option<String>,
    #[serde_as(as = "serde_with::TimestampSecondsWithFrac<f64>")]
    unix_timestamp: DateTime<Utc>,
    utc: String,
}

/// An aircraft. (Might not actually be an aircraft—could be a ground vehicle,
/// or a test beacon.)
#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Aircraft {
    /// ACAS Resolution Advisory (RA) information. See the
    /// [sprintACASInfoShort](https://github.com/wiedehopf/readsb/blob/939391f9e1c128bee7f061770ff3166ef2a475fb/json_out.c#L233)
    /// function in readsb for the format.
    pub acas_ra: Option<AcasRa>,

    /// The version of ADS-B the aircraft is using.
    #[serde(rename = "version")]
    pub adsb_version: Option<u8>,

    // The aircraft type. E.g. "C172".
    #[serde(rename = "t")]
    pub aircraft_type: Option<String>,

    #[serde(rename = "baro_rate")]
    pub barometric_vertical_rate: Option<i32>,

    #[serde(rename = "alt_baro")]
    pub barometric_altitude: Option<AltitudeOrGround>,

    pub calc_track: Option<u16>,

    #[serde(rename = "flight")]
    pub call_sign: Option<String>,

    #[serde(rename = "dbFlags", default)]
    pub database_flags: DatabaseFlags,

    pub dir: Option<f64>,

    /// Distance in nautical miles from the origin point.
    ///
    /// This field is only returned for queries that specify an origin point, e.g.
    /// https://adsbexchange.com/api/aircraft/v2/lat/34.1/lon/-118.1/dist/10
    #[serde(rename = "dst", default)]
    pub distance_nm: Option<f64>,

    pub emergency: Option<Emergency>,

    #[serde(rename = "category")]
    pub emitter_category: Option<String>,

    #[serde(rename = "alt_geom")]
    pub geometric_altitude: Option<i32>,

    #[serde(rename = "gva")]
    pub geometric_vertical_accuracy: Option<u8>,

    #[serde(rename = "geom_rate")]
    pub geometric_vertical_rate: Option<i16>,

    #[serde_as(as = "Option<TimestampSecondsWithFrac<f64>>")]
    #[serde(default)]
    #[serde(rename = "gpsOkBefore")]
    pub gps_ok_before: Option<DateTime<Utc>>,

    #[serde(rename = "gpsOkLat")]
    pub gps_ok_lat: Option<f64>,
    #[serde(rename = "gpsOkLon")]
    pub gps_ok_lon: Option<f64>,

    #[serde(rename = "gs")]
    pub ground_speed_knots: Option<f64>,

    pub hex: String,

    #[serde(rename = "ias")]
    pub indicated_air_speed_knots: Option<f64>,

    #[serde(rename = "alert", default, deserialize_with = "optional_bool_from_int")]
    pub is_alert: Option<bool>,

    #[serde(rename = "lastPosition")]
    last_position: Option<AgedPosition>,

    pub lat: Option<f64>,

    pub lon: Option<f64>,

    #[serde(rename = "mach")]
    pub mach: Option<f64>,

    #[serde(rename = "mag_heading")]
    pub magnetic_heading: Option<f64>,

    #[serde(rename = "type")]
    pub message_type: MessageType,

    #[serde(rename = "mlat")]
    pub mlat_fields: Option<Vec<String>>,

    pub nac_p: Option<u8>,

    pub nac_v: Option<u8>,

    pub nav_altitude_fms: Option<i32>,

    pub nav_altitude_mcp: Option<i32>,

    pub nav_heading: Option<f64>,

    pub nav_modes: Option<Vec<NavMode>>,

    pub nav_qnh: Option<f64>,

    pub nic: Option<u8>,

    pub nic_baro: Option<u8>,

    #[serde(rename = "messages")]
    pub num_messages: i32,

    #[serde(rename = "oat")]
    pub outside_air_temperature: Option<f64>,

    #[serde(rename = "rc")]
    pub radius_of_containment_meters: Option<u32>,

    #[serde(rename = "r")]
    pub registration: Option<String>,

    pub roll: Option<f64>,

    pub rr_lat: Option<f64>,

    pub rr_lon: Option<f64>,

    pub rssi: f64,

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
    pub total_air_temperature: Option<f64>,

    pub track: Option<f64>,

    pub track_rate: Option<f64>,

    #[serde(rename = "tas")]
    pub true_air_speed_knots: Option<f64>,

    pub true_heading: Option<f64>,

    #[serde(rename = "wd")]
    pub wind_direction: Option<u16>,

    #[serde(rename = "ws")]
    pub wind_speed: Option<u32>,
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
/// # let json_str = include_str!("../tests/v2-specimen-nearby.json");
/// let response = Response::from_str(&json_str).unwrap();
/// println!("Got {} aircraft", response.aircraft.len());
/// println!("1st aircraft:\n{:#?}", response.aircraft[0]);
/// ```
#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Response {
    /// The time this response was generated.
    #[serde(with = "ts_milliseconds")]
    pub now: DateTime<Utc>,

    /// The time this response was cached.
    #[serde(rename = "ctime")]
    #[serde(with = "ts_milliseconds")]
    pub cache_time: DateTime<Utc>,

    /// Server processing time the request required.
    #[serde(rename = "ptime")]
    #[serde_as(as = "DurationMilliSeconds<f64>")]
    pub processing_time: std::time::Duration,

    /// Total aircraft returned.
    #[serde(rename = "total")]
    pub num_aircraft: u64,

    /// Vector of all known aircraft.
    #[serde(rename = "ac", default, deserialize_with = "empty_aircraft_vec")]
    pub aircraft: Vec<Aircraft>,

    /// Error message, or if there isn't an error, "No error".
    #[serde(rename = "msg")]
    pub message: Option<String>,
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
           "ptime": 61,
           "total": 1,
           "ac": [{
                   "alert": 1,
                   "alt_baro": "ground",
                   "alt_geom": 123,
                   "baro_rate": 1350,
                   "calc_track": 43,
                   "category": "A0",
                   "dbFlags": 1,
                   "dir": 287.1,
                   "emergency": "none",
                   "flight": "N1234",
                   "geom_rate": 1000,
                   "gpsOkBefore": 1631836547.1,
                   "gpsOkLat": 40.0,
                   "gpsOkLon": -118.0,
                   "gs": 50,
                   "gva": 1,
                   "hex":"000000",
                   "ias": 45.1,
                   "lastPosition": {
                       "seen_pos": 120.4,
                       "lat": 4.0,
                       "lon": 4.1,
                       "rc": 186,
                       "nic": 8
                   },
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
                   "oat": -0,
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
                processing_time: std::time::Duration::new(0, 61000000),
                num_aircraft: 1,
                message: None,
                aircraft: vec![Aircraft {
                    acas_ra: None,
                    adsb_version: Some(2),
                    aircraft_type: Some("C172".to_string()),
                    barometric_vertical_rate: Some(1350),
                    barometric_altitude: Some(AltitudeOrGround::OnGround),
                    calc_track: Some(43),
                    call_sign: Some("N1234".to_string()),
                    database_flags: DatabaseFlags(DatabaseFlags::MILITARY),
                    dir: Some(287.1),
                    distance_nm: None,
                    emergency: Some(Emergency::None),
                    emitter_category: Some("A0".to_string()),
                    geometric_altitude: Some(123),
                    geometric_vertical_accuracy: Some(1),
                    geometric_vertical_rate: Some(1000),
                    gps_ok_before: Some(Utc.ymd(2021, 9, 16).and_hms_milli(23, 55, 47, 100)),
                    gps_ok_lat: Some(40.0),
                    gps_ok_lon: Some(-118.0),
                    ground_speed_knots: Some(50.0),
                    hex: "000000".to_string(),
                    indicated_air_speed_knots: Some(45.1),
                    is_alert: Some(true),
                    last_position: Some(AgedPosition {
                        seen_pos: std::time::Duration::new(120, 0),
                        lat: 4.0,
                        lon: 4.1,
                        nic: 8,
                        rc: 186,
                    }),
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
                    outside_air_temperature: Some(-0.0),
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
                    total_air_temperature: Some(-10.0),
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

    #[test]
    fn test_no_aircraft() {
        let input = r#"
        {
            "ac": null,
            "ctime": 1617722635672,
            "now": 1617722633900,
            "ptime": 61,
            "total": 0
        }"#;
        let response = Response::from_str(input).unwrap();
        assert_eq!(
            response,
            Response {
                now: Utc.ymd(2021, 4, 6).and_hms_milli(15, 23, 53, 900),
                cache_time: Utc.ymd(2021, 4, 6).and_hms_milli(15, 23, 55, 672),
                processing_time: std::time::Duration::new(0, 61000000),
                num_aircraft: 0,
                message: None,
                aircraft: vec![]
            }
        );
    }

    #[test]
    fn test_parse_with_acas() {
        let input = r#"
        {
            "ac": [
                {
                    "acas_ra": {
                        "ARA": "1100001",
                        "MTE": "0",
                        "RAC": "0000",
                        "RAT": "1",
                        "TTI": "01",
                        "advisory": "Clear of Conflict",
                        "advisory_complement": "",
                        "bytes": "E2C20026B7FF0C",
                        "threat_id_hex": "adffc3",
                        "unix_timestamp": 1617657530.42,
                        "utc": "2021-04-05 21:18:50.4"
                    },
                    "alert": 0,
                    "alt_baro": 1000,
                    "alt_geom": 1100,
                    "category": "A2",
                    "dir": 287.1,
                    "dst": 7.5,
                    "emergency": "none",
                    "flight": "N982MM  ",
                    "geom_rate": -1088,
                    "gs": 140.1,
                    "gva": 2,
                    "hex": "adb3b1",
                    "lat": 26.19104,
                    "lon": -80.24611,
                    "messages": 1769417,
                    "mlat": [],
                    "nac_p": 10,
                    "nac_v": 1,
                    "nav_altitude_mcp": 2016,
                    "nav_qnh": 1019.2,
                    "nic": 8,
                    "nic_baro": 1,
                    "r": "N982MM",
                    "rc": 186,
                    "rssi": -5,
                    "sda": 2,
                    "seen": 0,
                    "seen_pos": 0.1,
                    "sil": 3,
                    "sil_type": "perhour",
                    "spi": 0,
                    "squawk": "7402",
                    "t": "GALX",
                    "tisb": [],
                    "track": 87.95,
                    "type": "adsb_icao",
                    "version": 2
                }
            ],
            "ctime": 1616962493606,
            "now": 1616962493300,
            "ptime": 42,
            "total": 8893
        }"#;
        let response = Response::from_str(input).unwrap();
        let aircraft = response.aircraft[0].clone();
        assert_eq!(
            aircraft.acas_ra.unwrap(),
            AcasRa {
                advisory: "Clear of Conflict".to_string(),
                advisory_complement: "".to_string(),
                ara: "1100001".to_string(),
                bytes: "E2C20026B7FF0C".to_string(),
                mte: "0".to_string(),
                rac: "0000".to_string(),
                rat: "1".to_string(),
                threat_id_hex: Some("adffc3".to_string()),
                tti: "01".to_string(),
                unix_timestamp: Utc.ymd(2021, 4, 5).and_hms_milli(21, 18, 50, 420),
                utc: "2021-04-05 21:18:50.4".to_string(),
            }
        );
    }
}
