pub(crate) mod coordinates;
pub(crate) mod method;
pub(crate) mod parameters;
pub(crate) mod polar_circle_resolution;

pub use coordinates::Coordinates;
pub use method::{Method, prominent_methods};
pub use parameters::Parameters;
pub use polar_circle_resolution::{PolarCircleResolutionError, PolarCircleResolver};

/// Time adjustment for all prayer times.
///
/// The value is specified in minutes and can be either positive or negative.
#[derive(Default, PartialEq, Eq, Debug, Copy, Clone)]
#[expect(missing_docs, reason = "self-explanatory")]
pub struct TimeAdjustment {
    pub fajr: i64,
    pub sunrise: i64,
    pub dhuhr: i64,
    pub asr: i64,
    pub maghrib: i64,
    pub isha: i64,
}

/// Rule for approximating Fajr and Isha at high latitudes.
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[expect(missing_docs, reason = "self-explanatory")]
pub enum HighLatitudeRule {
    MiddleOfTheNight,
    SeventhOfTheNight,
    TwilightAngle,
}

/// Names of all obligatory prayers, sunrise, and Qiyam.
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum Prayer {
    /// Qiyam of yesterday
    QiyamYesterday,

    /// Fajr
    Fajr,

    /// Sunrise
    Sunrise,

    /// Dhuhr
    Dhuhr,

    /// Asr awwal
    ///
    /// Calculated for when shadow length equals object length.
    /// This is preferred by non-Hanafi schools.
    AsrAwwal,

    /// Asr thaani
    ///
    /// Calculated for when shadow length doubles object length.
    /// This is preferred by Hanafi school.
    AsrThaani,

    /// Maghrib
    Maghrib,

    /// Isha
    Isha,

    /// Qiyam
    Qiyam,
}

/// Error that arises when a query for current prayer is made with time outside of the day
/// the schedule has been calculated for.
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
#[expect(missing_docs, reason = "self-explanatory")]
pub enum TimeOutsideOfDate {
    Yesterday,
    Tomorrow,
}
