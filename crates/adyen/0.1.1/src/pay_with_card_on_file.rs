use crate::{currency::Currency, error::Error, Gateway};
use serde::{Deserialize, Serialize};

impl Gateway {
    pub async fn pay_with_card_on_file<'a>(
        &self,
        amount: u64,
        currency: &'a Currency,
        reference: &'a str,
        shopper_reference: &'a str,
        stored_payment_method_id: &'a str,
        return_url: &'a str,
        merchant_account: &'a str,
    ) -> Result<String, Error> {
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
            stored_payment_method_id: &'a str,
        }

        let payment_method = PaymentMethod {
            r#type: "scheme",
            stored_payment_method_id,
        };

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            amount: Amount<'a>,
            reference: &'a str,
            payment_method: PaymentMethod<'a>,
            shopper_reference: &'a str,
            shopper_interaction: &'a str,
            recurring_processing_model: &'a str,
            return_url: &'a str,
            merchant_account: &'a str,
        }

        let body = Request {
            amount,
            payment_method,
            reference,
            shopper_reference, // Min length: 3, Max length: 256
            shopper_interaction: "ContAuth",
            recurring_processing_model: "UnscheduledCardOnFile",
            return_url,
            merchant_account,
        };

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            psp_reference: String,
            // result_code: String,
        }

        let url = "https://checkout-test.adyen.com/v71/payments";
        let res: Response = self.post(url, &body).await?;

        Ok(res.psp_reference)
    }
}
