use crate::{error::Error, payment, Gateway};
use serde::Serialize;

impl Gateway {
    // https://docs.adyen.com/payment-methods/vipps/api-only/
    pub async fn set_redirect_result<'a>(
        &self,
        redirect_result: &'a str,
    ) -> Result<payment::Response, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Details<'a> {
            redirect_result: &'a str,
        }

        let details = Details { redirect_result };

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Request<'a> {
            details: Details<'a>,
        }

        let body = Request { details };

        let url = format!("{}/v71/payments/details", self.base_api_url);
        let res: payment::Response = self.post(&url, &body).await?;

        Ok(res)
    }
}
