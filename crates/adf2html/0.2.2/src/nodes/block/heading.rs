use serde::{Deserialize, Serialize};

use crate::{
    nodes::inline::inline_node::InlineNode, 
    ToHtml
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Heading {
    pub content: Option<Vec<InlineNode>>,
    #[serde(rename = "attrs")]
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub level: i8,
    pub local_id: Option<String>,
}

impl ToHtml for Heading {
    fn to_html(&self) -> String {
        let tag = format!("h{}", self.attributes.level);
        let id = self.attributes.local_id.as_ref().map(|id| format!(" id = {id}")).unwrap_or_default();

        let mut html = String::new();

        if let Some(content) = &self.content {
            html = content
                .iter()
                .map(|n| n.to_html())
                .collect();
        }

        format!(r#"<{tag}{id} style = "padding: 4px;">{html}</{tag}>"#)
    }
}
