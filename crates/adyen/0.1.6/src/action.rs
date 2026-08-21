use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct SchemeRedirectData {
    m_d: String,

    pa_req: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    term_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum Scheme {
    #[serde(rename = "redirect")]
    #[serde(rename_all = "camelCase")]
    Redirect {
        url: String,
        data: SchemeRedirectData,
        method: String,
    },

    #[serde(rename = "threeDS2")]
    #[serde(rename_all = "camelCase")]
    ThreeDS2 {
        payment_data: String,
        subtype: String,
        token: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "paymentMethodType")]
#[serde(rename_all = "lowercase")]
pub enum Action {
    #[serde(rename_all = "camelCase")]
    Scheme(Scheme),

    #[serde(rename_all = "camelCase")]
    Swish {
        qr_code_data: String,
        r#type: String,
        payment_data: String,
        url: String,
    },

    #[serde(rename_all = "camelCase")]
    Vipps {
        method: String,
        url: String,
        r#type: String,
    },
}
