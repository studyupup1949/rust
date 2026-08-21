//! https://developer.adobe.com/document-services/docs/apis/#tag/Assets/operation/asset.uploadpresignedurl

use reqwest::{Client, header::HeaderMap};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{ApiHttpRequest, Error};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUploadPresignedUri {
    /// An asset ID identifying an asset that is globally unique and never reused.
    #[serde(rename = "assetID")]
    pub asset_id: String,
    /// The URL used to upload the Resource directly to the cloud provider.
    pub upload_uri: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUploadPresignedUriParams {
    /// The content type of the file to be stored. For e.g: application/pdf.
    pub media_type: String,
}

impl ApiHttpRequest for GetUploadPresignedUri {
    type Response = Self;
    type Params = (HeaderMap, GetUploadPresignedUriParams);

    async fn send(base_url: &Url, params: Self::Params) -> Result<Self::Response, Error> {
        let url = base_url.join("/assets")?;
        let client = Client::new();
        let resp = client
            .post(url)
            .json(&params.1)
            .headers(params.0)
            .send()
            .await?;

        if resp.status().is_success() {
            let resp: Self = resp.json().await?;
            return Ok(resp);
        }

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(Error::ApiError(format!(
            "Status: {}, Response: {}",
            status, text
        )))
    }
}
