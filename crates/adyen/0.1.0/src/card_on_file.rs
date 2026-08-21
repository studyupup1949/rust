use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CardOnFile {
    pub holder_name: String,
    pub issuer_country: String,
    pub card_summary: String, // Last 4.
    pub expiry_date: String, // The expiry date on the card (M/yyyy).
    pub r#type: String, // visa, mc
    pub recurring_detail_reference: String,
}