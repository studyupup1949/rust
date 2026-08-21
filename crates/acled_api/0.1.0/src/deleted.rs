use crate::{response::DeletedData, Error, Where};

/// This struct is used for specifying the query parameters for the `deleted`
/// endpoint. See <https://apidocs.acleddata.com/deleted_endpoint.html#query-filters>.
///
/// All fields are optional and can be default initialized.
///
/// ```
/// use acled_api::{DeletedQuery, Where};
///
/// let query = DeletedQuery {
///   timestamp: Where::GreaterThanOrEqual(1710025200),
///   ..Default::default()
/// };
/// ```
#[derive(Default)]
pub struct DeletedQuery {
    pub id: Where<String>,
    pub timestamp: Where<u64>,
}
// NOTE: undocumented but event_date=2024-02-15 also works, so maybe more as well?

impl DeletedQuery {
    pub(crate) fn as_parameters(&self) -> Vec<(String, String)> {
        let DeletedQuery { id, timestamp } = self;

        let mut parameters = Vec::new();
        parameters.extend_from_slice(&id.as_parameters("event_id_cnty"));
        parameters.extend_from_slice(&timestamp.as_parameters("deleted_timestamp"));
        parameters
    }
}

/// An event returned by the `deleted` endpoint.
///
/// Descriptions based on <https://apidocs.acleddata.com/deleted_endpoint.html>
#[derive(Clone, Debug)]
pub struct DeletedEvent {
    /// An individual identifier by number and country acronym.
    ///
    /// Renamed from `event_id_cnty`.
    pub id: String,
    /// The unix timestamp when this data entry was deleted.
    ///
    /// Renamed from `deleted_timestamp`.
    pub timestamp: u64,
}

impl TryFrom<DeletedData> for DeletedEvent {
    type Error = Error;

    fn try_from(data: DeletedData) -> Result<Self, Self::Error> {
        Ok(DeletedEvent {
            id: data.event_id_cnty,
            timestamp: data
                .deleted_timestamp
                .parse()
                .map_err(|_| Error::ParseError("deleted_timestamp".into()))?,
        })
    }
}
