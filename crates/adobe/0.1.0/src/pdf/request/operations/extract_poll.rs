//! https://developer.adobe.com/document-services/docs/apis/#tag/Extract-PDF/operation/pdfoperations.extractpdf.jobstatus

use reqwest::{Client, Url, header::HeaderMap};
use serde::{Deserialize, Serialize};

use crate::{ApiHttpRequest, Error};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExtractJobStatus {
    #[serde(rename = "in progress")]
    InProgress,
    Done,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExtractPoll {
    /// Job Status
    pub status: ExtractJobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Asset>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExtractPollParams {
    pub job_id: String,
}

impl ApiHttpRequest for ExtractPoll {
    type Response = Self;
    type Params = (HeaderMap, ExtractPollParams);

    async fn send(base_url: &Url, params: Self::Params) -> Result<Self::Response, Error> {
        let url = base_url.join(&format!("/operation/extractpdf/{}/status", params.1.job_id))?;
        let client = Client::new();
        let resp = client.get(url).headers(params.0).send().await?;

        if resp.status().is_success() {
            let resp: Self = resp.json().await?;
            return Ok(resp);
        }

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        Err(Error::ApiError(format!(
            "Failed to poll Extract PDF job status: {} - {}",
            status, text
        )))
    }
}
