/// Parses version 1 of the ADS-B Exchange API JSON format.

use chrono::{DateTime, Utc};


// V1 fields: https://discord.com/channels/546776636352364576/568298837249359883/813770525137961000
// V2 fields: https://www.adsbexchange.com/version-2-api-wip/
// "alt": "0",
// "altt": "0",
// "call": "DAL438",
// "cou": "United States",
// "dst": "13.82",
// "from": "MEM Memphis United States",
// "galt": "0",
// "gnd": "1",
// "icao": "A4A484",
// "interested": "0",
// "lat": "33.947312",
// "lon": "-118.403363",
// "mil": "0",
// "mlat": "0",
// "opicao": "",
// "pos": "1",
// "postime": "1614057549011",
// "reg": "N399DA",
// "sat": "0",
// "spd": "0",
// "sqk": "0031",
// "talt": "",
// "tisb": "0",
// "to": "LAX Los Angeles United States",
// "trak": "",
// "trkh": "0",
// "trt": "5",
// "ttrk": "",
// "type": "B738",
// "vsi": "",
// "vsit": "0",
// "wtc": "2"

pub enum AltitudeType {
    StandardPressureAltitude,
    IndicatedAltitude,
}

pub enum TrackType {
    Heading,
    GroundTrack
}

pub enum TransponderType {
    Unknown,
    ModeS,
    AdsB {
        version: u8
    }
}

/// Represents an aircraft.
pub struct Aircraft {
    /// Altitude in feet.
    pub altitude: Option<i32>,
    /// The type of altitude transmitted by the aircraft.
    pub altitude_type: AltitudeType,
    /// The callsign of the aircraft.
    pub call_sign: Option<String>,
    /// The country that the aircraft is registered to.  Based on the ICAO hex
    /// code range the aircraft is broadcasting.
    pub country: String,
    /// Distance?
    pub distance: u32,
    /// The altitude adjusted for local air pressure, should be roughly the
    /// height above mean sea level.
    pub geometric_altitude: Option<i32>,
    /// True if the aircraft is on the ground.
    pub is_on_ground: bool,
    /// This is the six-digit hexadecimal identifier broadcast by the aircraft
    /// over the air in order to identify itself.
    pub icao: String,
    /// is the aircraft marked as interesting in the database.
    pub is_interesting: bool,
    /// The aircraft’s latitude over the ground.
    pub latitude: f32,
    /// The aircraft’s longitude over the ground.
    pub longitude: f32,
    /// True if the aircraft appears to be operated by the military. Based on
    /// certain range of ICAO hex codes that the aircraft broadcasts.
    pub is_military: bool,
    /// True if the latitude and longitude appear to have been calculated by an
    /// MLAT (multilateration) server and were not transmitted by the aircraft.
    pub is_mlat: bool,
    /// The ICAO code of the operator.  Non-exhaustive list here:
    /// <https://en.wikipedia.org/wiki/List_of_airline_codes>
    pub operator_icao: Option<String>,
    pub has_position: bool,
    /// The time (at UTC in JavaScript ticks, UNIX epoch format in milliseconds)
    /// that the position was last reported by the aircraft. This field is the
    /// time at which the aircraft was at the lat/long/altitude reported above
    pub position_time: DateTime<Utc>,
    ///  Aircraft registration number.  This is looked up via a database based
    ///  on the ICAO code.  This information is only as good as the database,
    ///  and is not pulled off the airwaves. It is not broadcast by the
    ///  aircraft.
    pub registration: Option<String>,
    /// True if the data has been received via a SatCom ACARS feed.
    pub is_satcom: bool,
    /// The ground speed in knots.
    pub speed: f32,
    /// Transponder code.
    pub squawk: Option<String>,
    /// The target altitude, in feet, set on the autopilot / FMS etc. Broadcast
    /// by the aircraft.
    pub target_altitude: Option<i32>,
    /// True if the last message received for the aircraft was from a [TIS-B
    /// source](https://www.thebalance.com/what-is-tis-b-282714).
    pub is_tisb: bool,
    /// Aircraft’s track angle across the ground clockwise from 0° north.
    pub track: Option<f32>,
    /// True if Trak is the aircraft’s heading, false if it’s the ground track.
    /// Default to ground track until told otherwise.
    pub track_type: TrackType,
    pub transponder_type: TransponderType,
    /// The track or heading currently set on the aircraft’s autopilot or FMS.
    /// Broadcast by the aircraft.
    pub target_track: Option<f32>,
    /// The aircraft model’s ICAO type code. This is looked up via a database
    /// based on the ICAO code.  This information is only as good as the
    /// database, and is not pulled off the airwaves. It is not broadcast by the
    /// aircraft.
    pub icao_type: Option<String>,
    /// Vertical speed in feet per minute. Broadcast by the aircraft.
    pub vertical_speed: i32,


}
