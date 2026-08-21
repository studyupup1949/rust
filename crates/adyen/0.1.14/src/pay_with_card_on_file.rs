use crate::{currency::Currency, error::Error, payment, Gateway};
use serde::Serialize;

impl Gateway {
    // https://docs.adyen.com/payment-methods/cards/custom-card-integration/#make-payment-with-token
    // https://docs.adyen.com/online-payments/tokenization/advanced-flow/#pay-with-a-token
    pub async fn pay_with_card_on_file<'a>(
        &self,
        amount: u64,
        currency: &'a Currency,
        reference: &'a str,
        shopper_reference: &'a str,
        stored_payment_method_id: &'a str,
        return_url: &'a str,
        merchant_account: &'a str,
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

        let url = format!("{}/v71/payments", self.base_api_url);
        let res: payment::Response = self.post(&url, &body).await?;

        Ok(res)
    }
}
