use std::net::IpAddr;

use chrono::Duration;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

use crate::{
    types::{DataWrapper, Report},
    Client,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    network: IpNetwork,
    #[serde(
        rename = "maxAgeInDays",
        serialize_with = "crate::types::serde_duration_to_days"
    )]
    max_age: Option<Duration>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(rename = "networkAddress")]
    address: IpAddr,
    netmask: IpAddr,
    min_address: IpAddr,
    max_address: IpAddr,
    num_possible_hosts: u32,
    #[serde(rename = "addressSpaceDesc")]
    address_space_description: String,
    reports: Vec<Report>,
}

impl Client {
    pub async fn check_block(
        &self,
        network: IpNetwork,
        max_age: Option<Duration>,
    ) -> crate::Result<Response> {
        let max_age = max_age.map(Into::into);

        let request = Request { network, max_age };

        let data: DataWrapper<Response> = self
            .get(self.base.join("check-block").unwrap(), request)
            .await?;

        Ok(data.data)
    }
}
