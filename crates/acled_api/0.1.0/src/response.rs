use crate::Error;
use serde::Deserialize;

/*
// XXX: The docs are lying, the status can be either a string or int.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Status {
    String(String),
    Int32(i32),
}
*/

#[derive(Deserialize, Debug)]
pub(crate) struct AcledData {
    pub event_id_cnty: String,
    pub event_date: String,
    pub timestamp: String,

    pub disorder_type: String,
    pub event_type: String,
    pub sub_event_type: String,

    pub country: String,
    pub region: String,
    pub admin1: String,

    pub latitude: String,
    pub longitude: String,

    pub notes: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct DeletedData {
    pub event_id_cnty: String,
    pub deleted_timestamp: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ErrorData {
    // status: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum Response<T> {
    Data {
        // status: Status,
        success: bool,
        count: u32,
        data: Vec<T>,
    },
    Error {
        // status: Status,
        success: bool,
        count: u32,
        error: ErrorData,
    },
}

impl<T> Response<T> {
    pub(crate) fn into<S: std::convert::TryFrom<T, Error = Error>>(self) -> Result<Vec<S>, Error> {
        match self {
            Self::Data {
                data,
                success,
                count,
            } => {
                assert!(success);
                assert_eq!(count as usize, data.len());
                data.into_iter().map(TryInto::try_into).collect()
            }
            Self::Error {
                success,
                count,
                error,
            } => {
                assert!(!success);
                assert_eq!(count, 0);
                Err(Error::APIError {
                    message: error.message,
                })
            }
        }
    }
}
