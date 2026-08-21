use serde::{Deserialize, Serialize};

use crate::ToHtml;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    #[serde(rename = "attrs")]
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub id: Option<String>,
    pub short_name: String,
    pub text: Option<String>,
}

impl ToHtml for Emoji {
    fn to_html(&self) -> String {
        let emoji_string = self.attributes.text.clone().unwrap_or(self.attributes.short_name.clone());
        format!(r#"<span style = "padding: 4px;">{emoji_string}</span>"#)
    }
}
