use serde::{Deserialize, Serialize};

use crate::{
    nodes::inline::inline_node::InlineNode, 
    ToHtml
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paragraph {
    pub content: Option<Vec<InlineNode>>,
    #[serde(rename = "attrs")]
    pub attributes: Option<Attributes>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub local_id: Option<String>,
}

impl ToHtml for Paragraph {
    fn to_html(&self) -> String {
        let mut html = String::from("<p>");

        if let Some(attributes) = &self.attributes {
            if let Some(local_id) = &attributes.local_id {
                html = format!("<p id = {local_id}>");
            }
        }

        if let Some(content) = &self.content {
            for inline_node in content {
                html.push_str(&inline_node.to_html());
            }
        }

        html.push_str("</p>");

        html
    }
}
