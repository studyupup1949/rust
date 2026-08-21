//! https://developer.adobe.com/document-services/docs/apis/#tag/OCR

use std::{fmt::Display, str::FromStr};

use reqwest::{
    Client, StatusCode, Url,
    header::{HeaderMap, LOCATION},
};
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::{ApiHttpRequest, Error};

#[derive(Clone, Debug)]
pub enum OcrLang {
    DaDK,
    LtLT,
    SlSI,
    ElGR,
    RuRU,
    EnUS,
    ZhHK,
    HuHU,
    EtEE,
    PtBR,
    UkUA,
    NbNO,
    PlPL,
    LvLV,
    FiFI,
    JaJP,
    EsES,
    BgBG,
    EnGB,
    CsCZ,
    MtMT,
    DeDE,
    HrHR,
    SkSK,
    SrSR,
    CaCA,
    MkMK,
    KoKR,
    DeCH,
    NlNL,
    ZhCN,
    SvSE,
    ItIT,
    NoNO,
    TrTR,
    FrFR,
    RoRO,
    IwIL,
}

impl Display for OcrLang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OcrLang::DaDK => "da-DK",
            OcrLang::LtLT => "lt-LT",
            OcrLang::SlSI => "sl-SI",
            OcrLang::ElGR => "el-GR",
            OcrLang::RuRU => "ru-RU",
            OcrLang::EnUS => "en-US",
            OcrLang::ZhHK => "zh-HK",
            OcrLang::HuHU => "hu-HU",
            OcrLang::EtEE => "et-EE",
            OcrLang::PtBR => "pt-BR",
            OcrLang::UkUA => "uk-UA",
            OcrLang::NbNO => "nb-NO",
            OcrLang::PlPL => "pl-PL",
            OcrLang::LvLV => "lv-LV",
            OcrLang::FiFI => "fi-FI",
            OcrLang::JaJP => "ja-JP",
            OcrLang::EsES => "es-ES",
            OcrLang::BgBG => "bg-BG",
            OcrLang::EnGB => "en-GB",
            OcrLang::CsCZ => "cs-CZ",
            OcrLang::MtMT => "mt-MT",
            OcrLang::DeDE => "de-DE",
            OcrLang::HrHR => "hr-HR",
            OcrLang::SkSK => "sk-SK",
            OcrLang::SrSR => "sr-SR",
            OcrLang::CaCA => "ca-CA",
            OcrLang::MkMK => "mk-MK",
            OcrLang::KoKR => "ko-KR",
            OcrLang::DeCH => "de-CH",
            OcrLang::NlNL => "nl-NL",
            OcrLang::ZhCN => "zh-CN",
            OcrLang::SvSE => "sv-SE",
            OcrLang::ItIT => "it-IT",
            OcrLang::NoNO => "no-NO",
            OcrLang::TrTR => "tr-TR",
            OcrLang::FrFR => "fr-FR",
            OcrLang::RoRO => "ro-RO",
            OcrLang::IwIL => "iw-IL",
        };

        write!(f, "{}", s)
    }
}

impl FromStr for OcrLang {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "da-DK" => Ok(OcrLang::DaDK),
            "lt-LT" => Ok(OcrLang::LtLT),
            "sl-SI" => Ok(OcrLang::SlSI),
            "el-GR" => Ok(OcrLang::ElGR),
            "ru-RU" => Ok(OcrLang::RuRU),
            "en-US" => Ok(OcrLang::EnUS),
            "zh-HK" => Ok(OcrLang::ZhHK),
            "hu-HU" => Ok(OcrLang::HuHU),
            "et-EE" => Ok(OcrLang::EtEE),
            "pt-BR" => Ok(OcrLang::PtBR),
            "uk-UA" => Ok(OcrLang::UkUA),
            "nb-NO" => Ok(OcrLang::NbNO),
            "pl-PL" => Ok(OcrLang::PlPL),
            "lv-LV" => Ok(OcrLang::LvLV),
            "fi-FI" => Ok(OcrLang::FiFI),
            "ja-JP" => Ok(OcrLang::JaJP),
            "es-ES" => Ok(OcrLang::EsES),
            "bg-BG" => Ok(OcrLang::BgBG),
            "en-GB" => Ok(OcrLang::EnGB),
            "cs-CZ" => Ok(OcrLang::CsCZ),
            "mt-MT" => Ok(OcrLang::MtMT),
            "de-DE" => Ok(OcrLang::DeDE),
            "hr-HR" => Ok(OcrLang::HrHR),
            "sk-SK" => Ok(OcrLang::SkSK),
            "sr-SR" => Ok(OcrLang::SrSR),
            "ca-CA" => Ok(OcrLang::CaCA),
            "mk-MK" => Ok(OcrLang::MkMK),
            "ko-KR" => Ok(OcrLang::KoKR),
            "de-CH" => Ok(OcrLang::DeCH),
            "nl-NL" => Ok(OcrLang::NlNL),
            "zh-CN" => Ok(OcrLang::ZhCN),
            "sv-SE" => Ok(OcrLang::SvSE),
            "it-IT" => Ok(OcrLang::ItIT),
            "no-NO" => Ok(OcrLang::NoNO),
            "tr-TR" => Ok(OcrLang::TrTR),
            "fr-FR" => Ok(OcrLang::FrFR),
            _ => Err(Error::Other(format!("Invalid OcrLang: {}", s))),
        }
    }
}

impl Serialize for OcrLang {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for OcrLang {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = OcrLang::from_str(&String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)?;

        Ok(s)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OcrType {
    SearchableImage,
    SearchableImageExact,
}

impl Display for OcrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OcrType::SearchableImage => "searchable_image",
            OcrType::SearchableImageExact => "searchable_image_exact",
        };

        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ocr {
    /// Job status URI for polling the results
    pub location: String,
    /// Job ID extracted from the location URI
    pub job_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrParams {
    #[serde(rename = "assetID")]
    pub asset_id: String,
    pub ocr_lang: OcrLang,
    pub ocr_type: OcrType,
}

impl ApiHttpRequest for Ocr {
    type Response = Self;
    type Params = (HeaderMap, OcrParams);

    async fn send(base_url: &Url, params: Self::Params) -> Result<Self::Response> {
        let url = base_url.join("/operation/ocr")?;
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

            return Ok(Ocr { location, job_id });
        }

        Err(Error::ApiError(format!(
            "Failed to initiate OCR operation (Status: {}): {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        )))
    }
}

// Given: https://pdf-services-ue1.adobe.io/operation/ocr/dHjsarTf3dsmJiFNc2kOTcWf93UBt805/status
// Return: dHjsarTf3dsmJiFNc2kOTcWf93UBt805
fn extract_job_id(uri: &str) -> Result<String> {
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
