use crate::{currency::Currency, error::Error, payment, BrowserInfo, Gateway};
use serde::Serialize;

impl Gateway {
    // https://docs.adyen.com/payment-methods/cards/custom-card-integration/#make-a-payment
    pub async fn pay_with_new_card_on_file<'a>(
        &self,
        amount: u64,
        currency: &'a Currency,
        reference: &'a str,
        shopper_reference: &'a str,
        encrypted_card_number: &'a str,
        encrypted_expiry_month: &'a str,
        encrypted_expiry_year: &'a str,
        encrypted_security_code: &'a str,
        holder_name: &'a Option<&'a str>,
        return_url: &'a str,
        merchant_account: &'a str,
        channel: &'a Option<&'a str>,
        browser_info: &'a Option<&'a BrowserInfo>,
        shopper_email: &'a Option<&'a str>,
        shopper_i_p: &'a Option<&'a str>,
        origin: &'a Option<&'a str>,
        three_d_s_preferred: bool,
    ) -> Result<payment::Response, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Amount<'a> {
            value: u64,
            currency: &'a str,
        }

        let amount = Amount {
            value: amount,
            currency: &currency.to_string(),
        };

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct PaymentMethod<'a> {
            r#type: &'a str,

            encrypted_card_number: &'a str,

            encrypted_expiry_month: &'a str,

            encrypted_expiry_year: &'a str,

            encrypted_security_code: &'a str,

            #[serde(skip_serializing_if = "Option::is_none")]
            holder_name: &'a Option<&'a str>,
        }

        let payment_method = PaymentMethod {
            r#type: "scheme",
            encrypted_card_number,
            encrypted_expiry_month,
            encrypted_expiry_year,
            encrypted_security_code,
            holder_name,
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
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            amount: Amount<'a>,

            reference: &'a str,

            payment_method: PaymentMethod<'a>,

            #[serde(skip_serializing_if = "Option::is_none")]
            authentication_data: Option<AuthenticationData>,

            shopper_reference: &'a str,

            shopper_interaction: &'a str,

            recurring_processing_model: &'a str,

            store_payment_method: bool,

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
                }),
                false => None,
            },
            reference,
            shopper_reference, // Min length: 3, Max length: 256
            shopper_interaction: "Ecommerce",
            recurring_processing_model: "UnscheduledCardOnFile",
            store_payment_method: true,
            return_url,
            merchant_account,
            shopper_email,
            shopper_i_p,
            channel,
            origin,
            browser_info,
        };

        let url = "https://checkout-test.adyen.com/v71/payments";
        let res: payment::Response = self.post(url, &body).await?;

        Ok(res)
    }
}
