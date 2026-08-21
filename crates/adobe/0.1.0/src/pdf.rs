pub mod request;

use std::str::FromStr;

use bytes::Bytes;
use reqwest::Url;
use reqwest::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, HeaderMap, HeaderName};

use crate::Result;
use crate::pdf::request::assets::get_upload_presigned_uri::{
    GetUploadPresignedUri, GetUploadPresignedUriParams,
};
use crate::pdf::request::operations::extract::{Extract, ExtractParams};
use crate::pdf::request::operations::extract_poll::{ExtractPoll, ExtractPollParams};
use crate::pdf::request::operations::ocr::{Ocr, OcrParams};
use crate::pdf::request::operations::ocr_poll::{OcrPoll, OcrPollParams};
use crate::{ApiHttpRequest, pdf::request::token::TokenParams};

use self::request::token::Token;

pub const PDF_SERVICES_UE1_URL: &str = "https://pdf-services-ue1.adobe.io";
pub const PDF_SERVICES_EW1_URL: &str = "https://pdf-services-ew1.adobe.io";

/// Adobe PDF Services Client
#[derive(Clone, Debug)]
pub struct PdfServicesClient {
    base_url: Url,
    access_token: String,
    client_id: String,
    _token: Token,
}

impl PdfServicesClient {
    /// Creates a new instance of the PDF Services Client
    pub async fn new(base_url: &str, client_id: &str, client_secret: &str) -> Result<Self> {
        let base_url = Url::from_str(base_url)?;
        let token = Token::send(
            &base_url,
            TokenParams {
                client_id: client_id.to_owned(),
                client_secret: client_secret.to_owned(),
            },
        )
        .await?;

        Ok(PdfServicesClient {
            access_token: token.access_token.clone(),
            client_id: client_id.to_owned(),
            base_url,
            _token: token,
        })
    }

    #[inline]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    fn base_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let value = format!("Bearer {}", self.access_token);
        let x_api_key = HeaderName::from_str("X-API-Key")?;

        headers.insert(AUTHORIZATION, value.parse()?);
        headers.insert(x_api_key, self.client_id.parse()?);

        Ok(headers)
    }

    /// Get the pre-signed URI to directly upload the content to the adobe's internal cloud provider.
    /// Output: Response will have the asset URI of created asset and pre-signed URI for uploading the content.
    /// Note: The pre-signed URI provided for uploading or downloading the content has an expiry of 1 hour.
    pub async fn get_upload_pre_signed_uri(
        &self,
        get_upload_pre_signed_uri_params: GetUploadPresignedUriParams,
    ) -> Result<GetUploadPresignedUri> {
        GetUploadPresignedUri::send(
            &self.base_url,
            (self.base_headers()?, get_upload_pre_signed_uri_params),
        )
        .await
    }

    /// Convenience method to upload file to the pre-signed URI
    pub async fn upload_pdf_file(&self, presigned_uri: Url, bytes: Bytes) -> Result<()> {
        let client = reqwest::Client::new();
        let resp = client
            .put(presigned_uri)
            .header(CONTENT_TYPE, "application/pdf")
            .header(CONTENT_LENGTH, bytes.len())
            .body(bytes)
            .send()
            .await?;

        if resp.status().is_success() {
            return Ok(());
        }

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(crate::Error::ApiError(format!(
            "Failed to upload PDF file: {} - {}",
            status, text
        )))
    }

    /// Perform OCR on PDF Document by providing specific language and type.
    pub async fn perform_ocr(&self, ocr_params: OcrParams) -> Result<Ocr> {
        Ocr::send(&self.base_url, (self.base_headers()?, ocr_params)).await
    }

    /// Poll the ocr job for completion
    pub async fn poll_ocr(&self, ocr_poll_params: OcrPollParams) -> Result<OcrPoll> {
        OcrPoll::send(&self.base_url, (self.base_headers()?, ocr_poll_params)).await
    }

    /// Extract PDF Content, Tables content and Tables/Figures renditions from a PDF document.
    /// Various available options are: (mutually exclusive)
    ///
    /// - Extract text with structure (headings, paragraphs, lists and footnotes)
    /// - Extract tables data as CSV or XLSX
    /// - Extract tables as images.
    /// - Extract bounding boxes for characters present in text blocks(paragraphs, list, headings)
    /// - Extract figures or images as PNG
    pub async fn extract_pdf(&self, extract_params: ExtractParams) -> Result<Extract> {
        Extract::send(&self.base_url, (self.base_headers()?, extract_params)).await
    }

    /// Poll the extract pdf job for completion
    pub async fn poll_extract_pdf(
        &self,
        extract_poll_params: ExtractPollParams,
    ) -> Result<ExtractPoll> {
        ExtractPoll::send(&self.base_url, (self.base_headers()?, extract_poll_params)).await
    }
}
