use crate::{error::Error, payment, Gateway};
use serde::Serialize;

impl Gateway {
    // https://docs.adyen.com/online-payments/3d-secure/native-3ds2/web-component/?tab=create-new-component_2#submit-authentication-result
    pub async fn set_payment_details<'a>(
        &self,
        three_d_s_result: &'a str,
    ) -> Result<payment::Response, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Details<'a> {
            three_d_s_result: &'a str,
        }

        let details = Details { three_d_s_result };

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
