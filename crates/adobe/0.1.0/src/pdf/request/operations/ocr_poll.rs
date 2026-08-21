//! https://developer.adobe.com/document-services/docs/apis/#tag/OCR/operation/pdfoperations.ocr.jobstatus

use reqwest::{Client, Url, header::HeaderMap};
use serde::{Deserialize, Serialize};

use crate::{ApiHttpRequest, Error};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OcrJobStatus {
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
pub struct OcrPoll {
    /// Job Status
    pub status: OcrJobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<Asset>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OcrPollParams {
    pub job_id: String,
}

impl ApiHttpRequest for OcrPoll {
    type Response = Self;
    type Params = (HeaderMap, OcrPollParams);

    async fn send(base_url: &Url, params: Self::Params) -> Result<Self::Response, Error> {
        let url = base_url.join(&format!("/operation/ocr/{}/status", params.1.job_id))?;
        let client = Client::new();
        let resp = client.get(url).headers(params.0).send().await?;

        if resp.status().is_success() {
            let resp: Self = resp.json().await?;
            return Ok(resp);
        }

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        Err(Error::ApiError(format!(
            "Failed to poll OCR job status: {} - {}",
            status, text
        )))
    }
}
