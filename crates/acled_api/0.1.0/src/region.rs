use strum::{Display, EnumString};

use crate::AsParameter;

/// Numeric codes for each region in ACLED data.
/// <https://apidocs.acleddata.com/acled_endpoint.html#regions>
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Display, EnumString)]
pub enum Region {
    /// Western Africa
    #[strum(to_string = "Western Africa")]
    WesternAfrica = 1,
    /// Middle Africa
    #[strum(to_string = "Middle Africa")]
    MiddleAfrica = 2,
    /// Eastern Africa
    #[strum(to_string = "Eastern Africa")]
    EasternAfrica = 3,
    /// Southern Africa
    #[strum(to_string = "Southern Africa")]
    SouthernAfrica = 4,
    /// Northern Africa
    #[strum(to_string = "Northern Africa")]
    NorthernAfrica = 5,
    /// South Asia
    #[strum(to_string = "South Asia")]
    SouthAsia = 7,
    /// Southeast Asia
    #[strum(to_string = "Southeast Asia")]
    SoutheastAsia = 9,
    /// Middle East
    #[strum(to_string = "Middle East")]
    MiddleEast = 11,
    /// Europe
    Europe = 12,
    /// Caucasus and Central Asia
    #[strum(to_string = "Caucasus and Central Asia")]
    CaucasusAndCentralAsia = 13,
    /// Central America
    #[strum(to_string = "Central America")]
    CentralAmerica = 14,
    /// South America
    #[strum(to_string = "South America")]
    SouthAmerica = 15,
    /// Caribbean
    Caribbean = 16,
    /// East Asia
    #[strum(to_string = "East Asia")]
    EastAsia = 17,
    /// North America
    #[strum(to_string = "North America")]
    NorthAmerica = 18,
    /// Oceania
    Oceania = 19,
    /// Antarctica
    Antarctica = 20,
}

impl AsParameter for Region {
    fn as_parameter(&self) -> String {
        // Note: The query strings use the region ID number.
        (*self as usize).to_string()
    }
}
