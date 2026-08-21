use std::net::IpAddr;

use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::{
    types::{DataWrapper, IpVersion, Report, UsageType},
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
    verbose: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(rename = "ipAddress")]
    address: IpAddr,
    is_public: bool,
    #[serde(deserialize_with = "crate::types::serde_ip_version_from_i32")]
    ip_version: IpVersion,
    /// Only present if a whitelist lookup was performed
    is_whitelisted: Option<bool>,
    abuse_confidence_score: i32,
    country_code: Option<String>,
    country_name: Option<String>,
    #[serde(deserialize_with = "crate::types::serde_usage_type_from_str")]
    usage_type: Vec<UsageType>,
    isp: String,
    domain: Option<String>,
    hostnames: Vec<String>,
    is_tor: bool,
    total_reports: u32,
    num_distinct_users: u32,
    /// TODO: This is a string in the API, but it's actually a date
    last_reported_at: Option<String>,
    reports: Option<Vec<Report>>,
}

impl Client {
    pub async fn check(
        &self,
        address: IpAddr,
        max_age: Option<Duration>,
        verbose: Option<bool>,
    ) -> crate::Result<Response> {
        let max_age = max_age.map(Into::into);

        let request = Request {
            address,
            max_age,
            verbose,
        };

        let data: DataWrapper<Response> =
            self.get(self.base.join("check").unwrap(), request).await?;

        Ok(data.data)
    }
}
