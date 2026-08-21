use crate::{amount, error::Error, Gateway};
use serde::{Deserialize, Serialize};

impl Gateway {
    // https://docs.adyen.com/payment-methods/apple-pay/api-only/
    pub async fn make_apple_pay_session<'a>(
        &self,
        country_code: &'a str,
        amount: &'a amount::Amount,
        channel: &'a str,
        display_name: &'a str,
        domain_name: &'a str,
        merchant_account: &'a str,
    ) -> Result<String, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct PaymentMethodsRequest<'a> {
            merchant_account: &'a str,

            country_code: &'a str,

            amount: &'a amount::Amount,

            channel: &'a str,
        }

        let body = PaymentMethodsRequest {
            merchant_account,
            country_code,
            amount,
            channel,
        };

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Configuration {
            merchant_id: String,
            // merchant_name: String,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PaymentMethod {
            r#type: String,

            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(default)]
            configuration: Option<Configuration>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PaymentMethodsResponse {
            payment_methods: Vec<PaymentMethod>,
        }

        let url = "https://checkout-test.adyen.com/v71/paymentMethods";
        let res: PaymentMethodsResponse = self.post(url, &body).await?;

        // Get merchant identifier.
        let merchant_identifier = res
            .payment_methods
            .iter()
            .find(|method| method.r#type == "applepay")
            .and_then(|method| method.configuration.as_ref())
            .map(|config| &config.merchant_id)
            .ok_or_else(|| Error::UnsupportedPaymentMethod)?;

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct SessionsRequest<'a> {
            display_name: &'a str,
            domain_name: &'a str,
            merchant_identifier: &'a str,
        }

        let body = SessionsRequest {
            display_name,
            domain_name,
            merchant_identifier,
        };

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct ApplePaySession {
            pub data: String,
        }

        let url = "https://checkout-test.adyen.com/v71/applePay/sessions";
        let res: ApplePaySession = self.post(url, &body).await?;

        Ok(res.data)
    }
}
