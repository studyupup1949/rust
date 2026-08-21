use crate::{amount, error::Error, payment, Gateway};
use serde::Serialize;

impl Gateway {
    // https://docs.adyen.com/payment-methods/apple-pay/api-only/
    pub async fn pay_with_apple_pay<'a>(
        &self,
        amount: &'a amount::Amount,
        apple_pay_token: &'a str,
        reference: &'a str,
        return_url: &'a str,
        merchant_account: &'a str,
    ) -> Result<payment::Response, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct PaymentMethod<'a> {
            r#type: &'a str,
            apple_pay_token: &'a str,
        }

        let payment_method = PaymentMethod {
            r#type: "applepay",
            apple_pay_token,
        };

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            amount: &'a amount::Amount,
            reference: &'a str,
            payment_method: PaymentMethod<'a>,
            return_url: &'a str,
            merchant_account: &'a str,
        }

        let body = Request {
            amount,
            payment_method,
            reference,
            return_url,
            merchant_account,
        };

        let url = format!("{}/v71/payments", self.base_api_url);
        let res: payment::Response = self.post(&url, &body).await?;

        Ok(res)
    }
}
