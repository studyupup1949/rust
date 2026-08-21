mod acled;
mod deleted;
mod region;
mod response;

use crate::response::{AcledData, DeletedData, Response};
use reqwest::Url;

pub use crate::acled::{AcledEvent, AcledQuery};
pub use crate::deleted::{DeletedEvent, DeletedQuery};
pub use crate::region::Region;
pub use chrono::NaiveDate;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP request failed")]
    ReqwestError(#[from] reqwest::Error),

    /// Error that was returned by one of the API endpoints.
    #[error("API returned an error: {message}")]
    APIError { message: String },

    #[error("API response could not be parsed: {0}")]
    ParseError(String),
}

/// Configuration options for the API call. Currently this
/// just includes the required `key` and `email` parameters.
pub struct Configuration {
    pub key: String,
    pub email: String,
}

trait AsParameter {
    fn as_parameter(&self) -> String;
}

impl AsParameter for NaiveDate {
    fn as_parameter(&self) -> String {
        self.to_string()
    }
}
impl AsParameter for String {
    fn as_parameter(&self) -> String {
        self.clone()
    }
}
impl AsParameter for u32 {
    fn as_parameter(&self) -> String {
        self.to_string()
    }
}
impl AsParameter for u64 {
    fn as_parameter(&self) -> String {
        self.to_string()
    }
}

/// This enum is used to specify the filter options for a specific
/// parameter in a query.
///
/// See also <https://apidocs.acleddata.com/generalities_section.html#query-types>
#[allow(private_bounds)]
pub enum Where<T: AsParameter> {
    /// This default options means the query should not use this parameter
    /// at all; i.e., it's not added to the query string.
    Unspecified,
    /// This will use what ever default query type (comparison) is configured
    /// for this parameter. (Usually `LIKE` or `=`)
    Matches(T),
    /// Exactly match the specific parameter. (Query type `=`)
    Equal(T),
    /// Look for values like the value. Can use `*` to match the wildcard.
    /// (Query type `LIKE`)
    Like(T),
    /// Numeric/date value is greater than.
    /// (Query type `>`)
    GreaterThan(T),
    /// Numeric/date value is greater than or equal.
    /// (undocumented query type `>=`)
    GreaterThanOrEqual(T),
    Between(T, T),
}

impl<T: AsParameter> Default for Where<T> {
    fn default() -> Self {
        Self::Unspecified
    }
}

#[allow(private_bounds)]
impl<T: AsParameter> Where<T> {
    fn as_parameters(&self, name: &str) -> Vec<(String, String)> {
        match self {
            Self::Unspecified => Vec::new(),
            Self::Matches(v) => vec![(name.to_owned(), v.as_parameter())],
            Self::Equal(v) => vec![
                (format!("{name}_where"), "=".to_owned()),
                (name.to_owned(), v.as_parameter()),
            ],
            Self::Like(v) => vec![
                (format!("{name}_where"), "LIKE".to_owned()),
                (name.to_owned(), v.as_parameter()),
            ],
            Self::GreaterThan(v) => vec![
                (format!("{name}_where"), ">".to_owned()),
                (name.to_owned(), v.as_parameter()),
            ],
            Self::GreaterThanOrEqual(v) => vec![
                (format!("{name}_where"), ">=".to_owned()),
                (name.to_owned(), v.as_parameter()),
            ],
            Self::Between(a, b) => vec![
                (format!("{name}_where"), "BETWEEN".to_owned()),
                (
                    name.to_owned(),
                    format!("{}|{}", a.as_parameter(), b.as_parameter()),
                ),
            ],
        }
    }
}

/// The default row limit of the ACLED API is 5000.
/// See <https://apidocs.acleddata.com/generalities_section.html#adjusting-the-limit-on-the-number-of-rows-returned>
static DEFAULT_LIMIT: usize = 5000;

/// The main entry point that can be used to query the different endpoints
/// provided by ACLED.
///
/// See also <https://apidocs.acleddata.com/>.
///
/// ```
/// use acled_api::{Api, Configuration};
/// let configuration = Configuration {
///   key: "XXXXX".into(),
///   email: "foo@example.com".into()
/// };
/// let api = Api::new(configuration);
/// ```
pub struct Api {
    config: Configuration,
    base: String,
}

impl Api {
    // Initially inspired by https://crates.io/crates/fastly-api

    pub fn new(config: Configuration) -> Api {
        let base = "https://api.acleddata.com".to_owned();
        Api { config, base }
    }

    /// Query the `acled` endpoint for events.
    ///
    /// See also <https://apidocs.acleddata.com/acled_endpoint.html>.
    pub fn get_acled(&self, query: &AcledQuery) -> Result<Vec<AcledEvent>, Error> {
        let parameters = query.as_parameters();

        let mut all_events = Vec::new();
        for page in 1.. {
            let response = self
                .query("acled", &parameters, page)?
                .json::<Response<AcledData>>()?;
            let events = response.into::<AcledEvent>()?;

            all_events.extend_from_slice(&events);
            // Note: For some strange reason, the API doesn't explicitly
            // indicate that we have to request another page.
            if events.len() != DEFAULT_LIMIT {
                return Ok(all_events);
            }
        }

        unreachable!()
    }

    /// Query the `deleted` endpoint for (deleted) events.
    ///
    /// See also <https://apidocs.acleddata.com/deleted_endpoint.html>.
    pub fn get_deleted(&self, query: &DeletedQuery) -> Result<Vec<DeletedEvent>, Error> {
        let parameters = query.as_parameters();

        let mut all_events = Vec::new();
        for page in 1.. {
            let response = self
                .query("deleted", &parameters, page)?
                .json::<Response<DeletedData>>()?;
            let events = response.into::<DeletedEvent>()?;

            all_events.extend_from_slice(&events);
            // Note: For some strange reason, the API doesn't explicitly
            // indicate that we have to request another page.
            if events.len() != DEFAULT_LIMIT {
                return Ok(all_events);
            }
        }

        unreachable!()
    }

    fn query(
        &self,
        endpoint: &str,
        parameters: &[(String, String)],
        page: u32,
    ) -> reqwest::Result<reqwest::blocking::Response> {
        let mut params = parameters.to_vec();
        params.push(("key".into(), self.config.key.clone()));
        params.push(("email".into(), self.config.email.clone()));
        if page > 1 {
            params.push(("page".into(), page.to_string()))
        }

        let url = format!("{}/{endpoint}/read", self.base);
        let url_with_query =
            Url::parse_with_params(&url, &params).expect("URL parsing should never fail");
        reqwest::blocking::get(url_with_query)
    }
}
