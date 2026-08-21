use std::net::IpAddr;

use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::{
    types::{DataWrapper, Report},
    Client,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    #[serde(rename = "ipAddress")]
    address: IpAddr,
    #[serde(
        rename = "maxAgeInDays",
        serialize_with = "crate::types::serde_duration_to_days"
    )]
    max_age: Option<Duration>,
    page: Option<u32>,
    per_page: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    total: u32,
    page: u32,
    count: u32,
    per_page: u32,
    last_page: u32,
    next_page_url: String,
    previous_page_url: String,
    #[serde(rename = "results")]
    reports: Vec<Report>,
}

impl Client {
    pub async fn reports(
        &self,
        address: IpAddr,
        max_age: Option<Duration>,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> crate::Result<Response> {
        let max_age = max_age.map(Into::into);

        let request = Request {
            address,
            max_age,
            page,
            per_page,
        };

        let data: DataWrapper<Response> = self
            .get(self.base.join("reports").unwrap(), request)
            .await?;

        Ok(data.data)
    }
}
