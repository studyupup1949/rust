//! A Rust client library for Adobe PDF Services API.
//!
//! This library provides an interface to interact with Adobe PDF Services API, allowing you to
//! perform various operations such as extracting content from PDFs, performing OCR, and more.
//!
//! # Example
//!
//! ```ignore
//! use bytes::Bytes;
//!
//! use adobe::pdf::PDF_SERVICES_UE1_URL;
//! use adobe::pdf::PdfServicesClient;
//! use adobe::pdf::request::assets::get_upload_presigned_uri::GetUploadPresignedUriParams;
//! use adobe::pdf::request::operations::extract::Extract;
//! use adobe::pdf::request::operations::extract::ExtractParams;
//! use adobe::pdf::request::operations::extract_poll::ExtractJobStatus;
//! use adobe::pdf::request::operations::extract_poll::ExtractPollParams;
//! use adobe::pdf::request::operations::ocr::OcrLang;
//! use adobe::pdf::request::operations::ocr::OcrParams;
//! use adobe::pdf::request::operations::ocr::OcrType;
//! use adobe::pdf::request::operations::ocr_poll::{OcrJobStatus, OcrPollParams};
//!
//! #[tokio::test]
//! async fn perform_ocr_on_pdf() {
//!     let pdf_bytes = tokio::fs::read("../../fixtures/sample2.pdf").await.unwrap();
//!     let pdf_bytes = Bytes::from(pdf_bytes);
//!
//!     // 0. Login to get a token.
//!     let pdfsc = PdfServicesClient::new(
//!         PDF_SERVICES_UE1_URL,
//!         "CLIENT ID",
//!         "CLIENT SECRET",
//!     )
//!     .await
//!     .unwrap();
//!
//!     // 1. Upload a PDF to get an asset ID.
//!     let upload_pre_signed_uri = pdfsc
//!         .get_upload_pre_signed_uri(GetUploadPresignedUriParams {
//!             media_type: String::from("application/pdf"),
//!         })
//!         .await
//!         .unwrap();
//!
//!     pdfsc
//!         .upload_pdf_file(upload_pre_signed_uri.upload_uri, pdf_bytes)
//!         .await
//!         .unwrap();
//!
//!     // 2. Extracts content from PDF
//!     let extract_pdf = pdfsc
//!         .extract_pdf(ExtractParams {
//!             asset_id: upload_pre_signed_uri.asset_id,
//!             get_char_bounds: false,
//!             include_styling: false,
//!             elements_to_extract: vec!["text".to_string()],
//!         })
//!         .await
//!         .unwrap();
//!
//!     assert!(!extract_pdf.location.is_empty());
//!
//!     let mut poll_result = None;
//!
//!     while poll_result.is_none() {
//!         let extract_poll = pdfsc
//!             .poll_extract_pdf(ExtractPollParams {
//!                 job_id: extract_pdf.job_id.clone(),
//!             })
//!             .await
//!             .unwrap();
//!
//!         if extract_poll.status == ExtractJobStatus::Failed {
//!             panic!("Extract job failed");
//!         }
//!
//!         if let Some(content) = extract_poll.content {
//!             poll_result = Some(content);
//!         } else {
//!             println!("Extract job status: {:?}", extract_poll.status);
//!             tokio::time::sleep(std::time::Duration::from_secs(2)).await;
//!         }
//!     }
//!
//!     panic!("{:?}", poll_result);
//! }
//! ```

use reqwest::{
    Url,
    header::{InvalidHeaderName, InvalidHeaderValue},
};
use thiserror::Error;
use url::ParseError;

pub mod pdf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("API Error: {0}")]
    ApiError(String),
    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Header value error: {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error("Header name error: {0}")]
    InvalidHeaderName(#[from] InvalidHeaderName),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] ParseError),
    #[error("Error: {0}")]
    Other(String),
}

#[allow(async_fn_in_trait)]
pub trait ApiHttpRequest {
    type Response;
    type Params;

    async fn send(base_url: &Url, params: Self::Params) -> Result<Self::Response>;
}
