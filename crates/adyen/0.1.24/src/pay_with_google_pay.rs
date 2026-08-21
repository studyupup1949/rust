use crate::{amount, error::Error, payment, BrowserInfo, Gateway};
use serde::Serialize;

impl Gateway {
    // https://docs.adyen.com/payment-methods/google-pay/api-only/
    pub async fn pay_with_google_pay<'a>(
        &self,
        amount: &'a amount::Amount,
        google_pay_token: &'a str,
        reference: &'a str,
        shopper_reference: &'a str,
        return_url: &'a str,
        channel: &'a Option<&'a str>,
        browser_info: &'a Option<&'a BrowserInfo>,
        shopper_email: &'a Option<&'a str>,
        shopper_i_p: &'a Option<&'a str>,
        origin: &'a Option<&'a str>,
        three_d_s_preferred: bool,
        merchant_account: &'a str,
    ) -> Result<payment::Response, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct PaymentMethod<'a> {
            r#type: &'a str,

            google_pay_token: &'a str,
        }

        let payment_method = PaymentMethod {
            r#type: "googlepay",
            google_pay_token,
        };

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ThreeDSRequestData {
            native_three_d_s: String,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct AuthenticationData {
            three_d_s_request_data: ThreeDSRequestData,

            attempt_authentication: String,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            amount: &'a amount::Amount,

            reference: &'a str,

            payment_method: PaymentMethod<'a>,

            #[serde(skip_serializing_if = "Option::is_none")]
            authentication_data: Option<AuthenticationData>,

            shopper_reference: &'a str,

            shopper_interaction: &'a str,

            return_url: &'a str,

            merchant_account: &'a str,

            #[serde(skip_serializing_if = "Option::is_none")]
            shopper_email: &'a Option<&'a str>,

            #[serde(skip_serializing_if = "Option::is_none")]
            shopper_i_p: &'a Option<&'a str>,

            #[serde(skip_serializing_if = "Option::is_none")]
            channel: &'a Option<&'a str>,

            #[serde(skip_serializing_if = "Option::is_none")]
            origin: &'a Option<&'a str>,

            #[serde(skip_serializing_if = "Option::is_none")]
            browser_info: &'a Option<&'a BrowserInfo>,
        }

        let body = Request {
            amount,
            payment_method,
            authentication_data: match three_d_s_preferred {
                true => Some(AuthenticationData {
                    three_d_s_request_data: ThreeDSRequestData {
                        native_three_d_s: "preferred".to_string(),
                    },
                    attempt_authentication: "always".to_string(),
                }),
                false => None,
            },
            reference,
            shopper_reference, // Min length: 3, Max length: 256
            shopper_interaction: "Ecommerce",
            return_url,
            merchant_account,
            shopper_email,
            shopper_i_p,
            channel,
            origin,
            browser_info,
        };

        let url = format!("{}/v71/payments", self.base_api_url);
        let res: payment::Response = self.post(&url, &body).await?;

        Ok(res)
    }
}
