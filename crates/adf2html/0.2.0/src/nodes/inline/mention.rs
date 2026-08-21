use serde::{Deserialize, Serialize};

use crate::ToHtml;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mention {
    #[serde(rename = "attrs")]
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub id: String,
    pub access_level: Option<AccessLevel>,
    pub text: Option<String>,
    pub user_type: Option<UserType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AccessLevel {
    None,
    Site,
    Application, 
    Container,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UserType {
    Default,
    Special,
    App,
}

impl ToHtml for Mention {
    fn to_html(&self) -> String {
        let mention_string = self.attributes.text.clone().unwrap_or_default();
        format!(r#"<span style = "padding: 4px;">{mention_string}</span>"#)
    }
}
