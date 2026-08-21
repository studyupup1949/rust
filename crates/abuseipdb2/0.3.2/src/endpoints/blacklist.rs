use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::{types::IpVersion, Client};

fn serde_option_ip_version_to_i32<S>(
    version: &Option<IpVersion>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match version {
        Some(version) => crate::types::serde_ip_version_to_i32(version, serializer),
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    confidence_minimum: u32,
    limit: Option<u32>,
    only_countries: Option<String>,
    except_countries: Option<String>,
    #[serde(serialize_with = "serde_option_ip_version_to_i32")]
    ip_version: Option<IpVersion>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    meta: Meta,
    #[serde(rename = "data")]
    entries: Vec<Blacklist>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    /// TODO: This is a string in the API, but it's actually a date
    generated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blacklist {
    #[serde(rename = "ipAddress")]
    address: IpAddr,
    abuse_confidence_score: u32,
    /// TODO: This is a string in the API, but it's actually a date
    last_reported_at: String,
}

impl Client {
    pub async fn blacklist(
        &self,
        confidence_minimum: u32,
        limit: Option<u32>,
        only_countries: Option<Vec<String>>,
        except_countries: Option<Vec<String>>,
        ip_version: Option<IpVersion>,
    ) -> crate::Result<Response> {
        let request = Request {
            confidence_minimum,
            limit,
            only_countries: only_countries.map(|v| v.join(",")),
            except_countries: except_countries.map(|v| v.join(",")),
            ip_version,
        };

        self.get(self.base.join("blacklist").unwrap(), request)
            .await
    }
}
