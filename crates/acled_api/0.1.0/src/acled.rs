use crate::region::Region;
use crate::response::AcledData;
use crate::{Error, Where};
use chrono::NaiveDate;

/// This struct is used for specifying the query parameters for the `acled`
/// endpoint. See <https://apidocs.acleddata.com/acled_endpoint.html#query-filters>.
///
/// All fields are optional and can be default initialized.
///
/// The following query will find all events from Afghanistan since 2022.
/// ```
/// use acled_api::{AcledQuery, Where};
///
/// let query = AcledQuery {
///   country: Where::Matches("Afghanistan".into()),
///   year: Where::GreaterThanOrEqual(2022),
///   ..Default::default()
/// };
/// ```
#[derive(Default)]
pub struct AcledQuery {
    pub country: Where<String>,
    pub id: Where<String>,
    pub year: Where<u32>,
    pub region: Where<Region>,
    pub date: Where<NaiveDate>,
    pub timestamp: Where<u64>,
}

impl AcledQuery {
    pub(crate) fn as_parameters(&self) -> Vec<(String, String)> {
        let AcledQuery {
            country,
            id,
            year,
            region,
            date,
            timestamp,
        } = self;

        let mut parameters = Vec::new();
        parameters.extend_from_slice(&country.as_parameters("country"));
        parameters.extend_from_slice(&id.as_parameters("event_id_cnty"));
        parameters.extend_from_slice(&year.as_parameters("year"));
        parameters.extend_from_slice(&region.as_parameters("region"));
        parameters.extend_from_slice(&date.as_parameters("event_date"));
        parameters.extend_from_slice(&timestamp.as_parameters("timestamp"));
        parameters
    }
}

/// An event returned by the `acled` endpoint.
///
/// Descriptions based on <https://apidocs.acleddata.com/acled_endpoint.html>
#[derive(Clone, Debug)]
pub struct AcledEvent {
    /// A unique alphanumeric event identifier by number and country acronym.
    /// This identifier remains constant even when the event details are updated.
    ///
    /// Renamed from `event_id_cnty`.
    pub id: String,
    /// An automatically generated Unix timestamp that represents the exact date
    /// and time an event was last uploaded to the ACLED API.
    pub timestamp: u64,
    /// The date on which the event took place.
    ///
    /// Renamed from `event_date`.
    pub date: NaiveDate,
    /// The type of event; further specifies the nature of the event.
    /// Followed by the subcategory of the event type.
    ///
    /// Consist of the renamed `event_type` and `sub_event_type`.
    pub event_type: (String, String),
    /// The disorder category an event belongs to.
    pub disorder_type: String,
    /// The region of the world where the event took place.
    pub region: Region,
    /// The country or territory in which the event took place.
    pub country: String,
    /// The sub-national administrative region
    pub administrative_region: String,

    pub latitude: f64,
    pub longitude: f64,

    /// A short description of the event.
    ///
    /// Renamed from `notes`.
    pub note: String,
}

impl TryFrom<AcledData> for AcledEvent {
    type Error = Error;

    fn try_from(data: AcledData) -> Result<Self, Self::Error> {
        Ok(AcledEvent {
            id: data.event_id_cnty,
            date: NaiveDate::parse_from_str(&data.event_date, "%Y-%m-%d")
                .map_err(|_| Error::ParseError("event_date".into()))?,
            timestamp: data
                .timestamp
                .parse()
                .map_err(|_| Error::ParseError("timestamp".into()))?,
            event_type: (data.event_type, data.sub_event_type),
            disorder_type: data.disorder_type,
            region: data
                .region
                .parse()
                .map_err(|_| Error::ParseError("region".into()))?,
            administrative_region: data.admin1,
            country: data.country,
            latitude: data
                .latitude
                .parse()
                .map_err(|_| Error::ParseError("latitude".into()))?,
            longitude: data
                .longitude
                .parse()
                .map_err(|_| Error::ParseError("longitude".into()))?,
            note: data.notes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_test() {
        let query = AcledQuery {
            country: Where::Matches("Germany".into()),
            ..Default::default()
        };
        assert_eq!(
            query.as_parameters(),
            vec![("country".into(), "Germany".into())]
        );

        let query = AcledQuery {
            id: Where::Matches("GER-123".into()),
            ..Default::default()
        };
        assert_eq!(
            query.as_parameters(),
            vec![("event_id_cnty".into(), "GER-123".into())]
        );

        let query = AcledQuery {
            region: Where::Matches(Region::MiddleAfrica),
            date: Where::GreaterThan(NaiveDate::from_ymd_opt(2024, 02, 28).unwrap()),
            ..Default::default()
        };
        assert_eq!(
            query.as_parameters(),
            vec![
                ("region".into(), "2".into()),
                ("event_date_where".into(), ">".into()),
                ("event_date".into(), "2024-02-28".into())
            ]
        );
    }
}
