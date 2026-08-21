use rust_decimal::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;
mod error;
pub use error::Error;
mod currency;
pub use currency::Currency;
mod action;
pub use action::Action;
mod browser_info;
pub use browser_info::BrowserInfo;
mod make_apple_pay_session;
mod pay_with_apple_pay;
mod pay_with_card_on_file;
mod pay_with_google_pay;
mod pay_with_new_card_on_file;
mod pay_with_swish;
mod pay_with_vipps;
mod payment;
mod refund;
mod set_payment_details;
pub mod webhook;
pub use webhook::Webhook;
mod amount;
pub use amount::Amount;
pub use payment::RefusalReason;
pub mod prelude {
    pub use super::{
        action::{Action, Scheme as SchemeAction, SchemeRedirectData},
        browser_info::BrowserInfo,
        payment::{RefusalReason, Response},
        Currency, Environment, Error, Gateway,
    };
}

pub enum Environment {
    Test { api_key: String },
    Live { api_key: String, url_prefix: String },
}

pub struct Gateway {
    client: reqwest::Client,
    _environment: Environment,
    base_api_url: String,
}

pub fn convert_decimal_into_minor_units<'a>(
    amount: &'a Decimal,
    currency: &'a Currency,
) -> Result<u64, Error> {
    let decimals = match currency {
        Currency::NOK => 2,
        Currency::SEK => 2,
        Currency::DKK => 2,
        Currency::ISK => 0,
        Currency::GBP => 2,
        Currency::EUR => 2,
    };

    // Get minor units from decimal.
    let minor_units_modifier = Decimal::from(10i32.pow(decimals));
    let amount = amount * minor_units_modifier;

    match amount.to_u64() {
        Some(a) => Ok(a),
        None => {
            return Err(Error::ConversionError(format!(
                "could not convert \"{}\" to u64",
                amount
            )))
        }
    }
}

impl Gateway {
    pub fn new(environment: Environment, timeout: Option<Duration>) -> Result<Gateway, Error> {
        let base_api_url = match &environment {
            Environment::Test { .. } => format!("https://checkout-test.adyen.com"),
            Environment::Live { url_prefix, .. } => {
                format!(
                    "https://{}-checkout-live.adyenpayments.com/checkout",
                    url_prefix
                )
            }
        };

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Accept",
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        let api_key = match &environment {
            Environment::Test { api_key } | Environment::Live { api_key, .. } => api_key,
        };

        let x_api_key_header_value = format!("{}", api_key);
        let x_api_key_header = match reqwest::header::HeaderValue::from_str(&x_api_key_header_value)
        {
            Ok(header) => header,
            Err(err) => {
                return Err(Error::Unspecified(format!(
                    "could not create auth header ({})",
                    err.to_string()
                )))
            }
        };

        headers.insert("x-API-key", x_api_key_header);

        let timeout = match timeout {
            Some(t) => t,
            None => Duration::new(60, 0),
        };

        let client = match reqwest::ClientBuilder::new()
            .default_headers(headers)
            .https_only(true)
            .timeout(timeout)
            .build()
        {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::Unspecified(format!(
                    "could not create reqwest client ({})",
                    err.to_string()
                )))
            }
        };

        Ok(Gateway {
            client,
            _environment: environment,
            base_api_url,
        })
    }

    async fn post<'a, T: DeserializeOwned>(
        &self,
        url: &str,
        body: impl Serialize,
    ) -> Result<T, Error> {
        let res = match self.client.post(url).json(&body).send().await {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::NetworkError(format!(
                    "could not send request ({})",
                    err.to_string()
                )))
            }
        };

        let status = res.status();
        let text = res
            .text()
            .await
            .unwrap_or_else(|_| String::from("Could not retrieve body text."));

        let status = status.as_u16();
        if status < 200 || status >= 300 {
            #[derive(Deserialize, Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            struct ApiError {
                pub status: u16,

                pub error_code: String,

                pub message: String,

                pub error_type: String,

                #[serde(default)]
                pub psp_reference: Option<String>,
            }

            let api_error: ApiError = serde_json::from_str(&text).map_err(|err| {
                Error::Unspecified(format!(
                    "could not parse api error from '{}' ({})",
                    text, err
                ))
            })?;

            return Err(Error::ApiError(error::ApiError::Other {
                status: api_error.status,
                error_code: api_error.error_code,
                message: api_error.message,
                error_type: api_error.error_type,
                psp_reference: api_error.psp_reference,
            }));
        }

        let body: T = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::SerializationError(format!(
                    "could not deserialize response ({}): {}",
                    err, text
                )))
            }
        };
        Ok(body)
    }

    // async fn get<'a, T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
    //     let res = match self.client.get(url).send().await {
    //         Ok(r) => r,
    //         Err(err) => {
    //             return Err(Error::NetworkError(format!(
    //                 "could not send request ({})",
    //                 err
    //             )))
    //         }
    //     };

    //     let status = res.status();
    //     let text = res
    //         .text()
    //         .await
    //         .unwrap_or_else(|_| String::from("Could not retrieve body text."));

    //     if status.as_u16() < 200 || status.as_u16() >= 300 {
    //         #[derive(Deserialize, Debug, Clone)]
    //         #[serde(rename_all = "camelCase")]
    //         struct ApiError {
    //             pub meta: Option<Meta>,
    //         }

    //         let api_error: ApiError =
    //             serde_json::from_str(&text).unwrap_or_else(|_| ApiError { meta: None });

    //         let meta = match api_error.meta {
    //             Some(m) => m,
    //             None => {
    //                 return Err(Error::ApiError(
    //                     status.as_u16().to_string(),
    //                     "unknown".to_string(),
    //                     None,
    //                     None,
    //                     text,
    //                 ))
    //             }
    //         };

    //         let action = match meta.action {
    //             Some(m) => m,
    //             None => {
    //                 return Err(Error::ApiError(
    //                     status.as_u16().to_string(),
    //                     "unknown".to_string(),
    //                     None,
    //                     None,
    //                     text,
    //                 ))
    //             }
    //         };

    //         let code = match action.code {
    //             Some(m) => m,
    //             None => {
    //                 return Err(Error::ApiError(
    //                     status.as_u16().to_string(),
    //                     "unknown".to_string(),
    //                     None,
    //                     None,
    //                     text,
    //                 ))
    //             }
    //         };

    //         let source = match action.source {
    //             Some(m) => m,
    //             None => {
    //                 return Err(Error::ApiError(
    //                     status.as_u16().to_string(),
    //                     "unknown".to_string(),
    //                     None,
    //                     None,
    //                     text,
    //                 ))
    //             }
    //         };

    //         let (enduser_message, merchant_message) = match meta.message {
    //             Some(m) => (m.enduser, m.merchant),
    //             None => (None, None),
    //         };

    //         return Err(Error::ApiError(
    //             code,
    //             source,
    //             enduser_message,
    //             merchant_message,
    //             text,
    //         ));
    //     }

    //     let body: T = match serde_json::from_str(&text) {
    //         Ok(r) => r,
    //         Err(err) => {
    //             return Err(Error::SerializationError(format!(
    //                 "could not deserialize response ({}): {}",
    //                 err, text
    //             )))
    //         }
    //     };
    //     Ok(body)
    // }
}
