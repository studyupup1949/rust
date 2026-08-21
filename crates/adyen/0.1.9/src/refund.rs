use crate::{currency::Currency, error::Error, Gateway};
use serde::{Deserialize, Serialize};

impl Gateway {
    // https://docs.adyen.com/online-payments/refund/#refund-cancel-or-reverse-a-payment
    pub async fn refund<'a>(
        &self,
        amount: u64,
        currency: &'a Currency,
        reference: &'a str,
        psp_reference: &'a str,
        merchant_account: &'a str,
    ) -> Result<(), Error> {
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
        struct Request<'a> {
            amount: Amount<'a>,
            reference: &'a str,
            merchant_account: &'a str,
        }

        let body = Request {
            amount,
            reference,
            merchant_account,
        };

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {}

        let url = format!(
            "https://checkout-test.adyen.com/v71/payments/{}/refunds",
            psp_reference
        );
        let _res: Response = self.post(&url, &body).await?;

        Ok(())
    }
}
