//! https://developer.adobe.com/document-services/docs/apis/#tag/Extract-PDF/operation/pdfoperations.extractpdf

use reqwest::{
    Client, StatusCode, Url,
    header::{HeaderMap, LOCATION},
};
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::{ApiHttpRequest, Error};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Extract {
    /// Job status URI for polling the results
    pub location: String,
    /// Job ID extracted from the location URI
    pub job_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractParams {
    #[serde(rename = "assetID")]
    pub asset_id: String,
    pub get_char_bounds: bool,
    pub include_styling: bool,
    pub elements_to_extract: Vec<String>,
}

impl ApiHttpRequest for Extract {
    type Response = Self;
    type Params = (HeaderMap, ExtractParams);

    async fn send(base_url: &Url, params: Self::Params) -> Result<Self::Response> {
        let url = base_url.join("/operation/extractpdf")?;
        let client = Client::new();
        let resp = client
            .post(url)
            .headers(params.0)
            .json(&params.1)
            .send()
            .await?;

        if resp.status() == StatusCode::CREATED {
            let location = resp
                .headers()
                .get(LOCATION)
                .ok_or_else(|| Error::ApiError("Missing Location header".to_string()))?
                .to_str()
                .map_err(|e| Error::ApiError(format!("Invalid Location header: {}", e)))?
                .to_string();
            let job_id = extract_job_id(&location)?;

            return Ok(Extract { location, job_id });
        }

        Err(Error::ApiError(format!(
            "Failed to initiate Extract PDF operation (Status: {}): {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        )))
    }
}

/// Given: https://pdf-services-ue1.adobe.io/operation/ocr/dHjsarTf3dsmJiFNc2kOTcWf93UBt805/status
///
/// Return: dHjsarTf3dsmJiFNc2kOTcWf93UBt805
pub fn extract_job_id(uri: &str) -> Result<String> {
    let mut parts: Vec<&str> = uri.split('/').collect();

    parts.pop(); // Remove "status"

    let job_id = parts
        .pop()
        .ok_or_else(|| Error::ApiError("Invalid URI format".to_string()))?;

    Ok(job_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_job_id() {
        let uri = "https://pdf-services-ue1.adobe.io/operation/ocr/dHjsarTf3dsmJiFNc2kOTcWf93UBt805/status";
        let job_id = extract_job_id(uri).unwrap();
        assert_eq!(job_id, "dHjsarTf3dsmJiFNc2kOTcWf93UBt805");
    }
}
