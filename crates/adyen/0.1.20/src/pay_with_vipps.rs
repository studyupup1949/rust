use crate::{currency::Currency, error::Error, payment, Gateway};
use serde::Serialize;

impl Gateway {
    // https://docs.adyen.com/payment-methods/swish/api-only/
    pub async fn pay_with_vipps<'a>(
        &self,
        amount: u64,
        currency: &'a Currency,
        reference: &'a str,
        return_url: &'a str,
        merchant_account: &'a str,
        channel: &'a str,
        telephone_number: &'a Option<&'a str>,
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

            #[serde(skip_serializing_if = "Option::is_none")]
            telephone_number: &'a Option<&'a str>,
        }

        let payment_method = PaymentMethod {
            r#type: "vipps",
            telephone_number,
        };

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            amount: Amount<'a>,
            reference: &'a str,
            payment_method: PaymentMethod<'a>,
            return_url: &'a str,
            merchant_account: &'a str,
            channel: &'a str,
        }

        let body = Request {
            amount,
            payment_method,
            reference,
            return_url,
            merchant_account,
            channel,
        };

        let url = format!("{}/v71/payments", self.base_api_url);
        let res: payment::Response = self.post(&url, &body).await?;

        Ok(res)
    }
}
