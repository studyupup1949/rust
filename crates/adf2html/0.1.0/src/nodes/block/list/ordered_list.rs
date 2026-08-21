use serde::{Deserialize, Serialize};

use crate::ToHtml;
use super::list_item::ListItem;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderedList {
    pub content: Vec<ListItem>,
    #[serde(rename = "attrs")]
    pub attributes: Option<Attributes>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub order: Option<u32>,
}

impl ToHtml for OrderedList {
    fn to_html(&self) -> String {
        let html = self.content
            .iter()
            .map(|item| item.to_html())
            .collect::<String>();

        format!(r#"<ol style = "padding: 4px;">{html}</ol>"#)
    }
}
