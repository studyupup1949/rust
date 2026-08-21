use crate::{card_on_file::CardOnFile, currency::Currency, error::Error, Gateway};
use serde::{Deserialize, Serialize};

impl Gateway {
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
    ) -> Result<(String, CardOnFile), Error> {
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
        struct Request<'a> {
            amount: Amount<'a>,
            reference: &'a str,
            payment_method: PaymentMethod<'a>,
            shopper_reference: &'a str,
            shopper_interaction: &'a str,
            recurring_processing_model: &'a str,
            store_payment_method: bool,
            return_url: &'a str,
            merchant_account: &'a str,
        }

        let body = Request {
            amount,
            payment_method,
            reference,
            shopper_reference, // Min length: 3, Max length: 256
            shopper_interaction: "Ecommerce",
            recurring_processing_model: "UnscheduledCardOnFile",
            store_payment_method: true,
            return_url,
            merchant_account,
        };

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AdditionalData {
            card_holder_name: String,
            issuer_country: String,
            card_summary: String,
            expiry_date: String,    // The expiry date on the card (M/yyyy).
            payment_method: String, // visa, mastercard, etc.

            #[serde(rename = "recurring.recurringDetailReference")]
            recurring_detail_reference: String,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            additional_data: AdditionalData,
            psp_reference: String,
            // result_code: String,
            // merchant_reference: String,
            // refusal_reason: String,
            // refusal_reason_code: u8,
        }

        let url = "https://checkout-test.adyen.com/v71/payments";
        let res: Response = self.post(url, &body).await?;

        // Make a card on file object.
        let card_on_file = CardOnFile {
            holder_name: res.additional_data.card_holder_name,
            issuer_country: res.additional_data.issuer_country,
            card_summary: res.additional_data.card_summary,
            expiry_date: res.additional_data.expiry_date,
            r#type: res.additional_data.payment_method,
            recurring_detail_reference: res.additional_data.recurring_detail_reference,
        };

        Ok((res.psp_reference, card_on_file))
    }
}
