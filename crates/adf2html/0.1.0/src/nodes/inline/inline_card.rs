use serde::{Deserialize, Serialize};

use crate::ToHtml;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineCard {
    #[serde(rename = "attrs")]
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub data: Option<String>,
    pub url: Option<String>,
}

impl ToHtml for InlineCard {
    fn to_html(&self) -> String {
        format!(r#"<a style = "padding: 4px;" href = "{0}">{0}</a>"#, self.attributes.url.clone().unwrap_or_default())
    }
}
