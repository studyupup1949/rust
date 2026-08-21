use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// serde Option datetime to string
fn serde_option_datetime_to_string<S>(
    datetime: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match datetime {
        Some(datetime) => crate::types::serde_datetime_to_string(datetime, serializer),
        None => serializer.serialize_none(),
    }
}

use crate::{
    types::{Category, DataWrapper},
    Client,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request<'a> {
    #[serde(rename = "ip")]
    address: &'a IpAddr,
    #[serde(serialize_with = "crate::types::serde_categories_to_string")]
    categories: &'a [Category],
    comment: Option<&'a str>,
    #[serde(serialize_with = "serde_option_datetime_to_string")]
    timestamp: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(rename = "ipAddress")]
    address: IpAddr,
    abuse_confidence_score: u32,
}

impl Client {
    pub async fn report(
        &self,
        address: &IpAddr,
        categories: &[Category],
        comment: Option<&str>,
        timestamp: Option<DateTime<Utc>>,
    ) -> crate::Result<Response> {
        let request = Request {
            address,
            categories,
            comment,
            timestamp,
        };

        let data: DataWrapper<Response> = self
            .post(self.base.join("report").unwrap(), request)
            .await?;

        Ok(data.data)
    }
}
