use serde::{Deserialize, Serialize};

use crate::{
    nodes::block::top_level_block_nodes::TopLevelBlockNode, 
    ToHtml,
    TIMEZONE
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub version: u8,
    #[serde(rename = "type")]
    pub kind: String,
    pub content: Vec<TopLevelBlockNode>,
}

impl Document {
    pub fn to_html(&self, timezone: Option<chrono_tz::Tz>) -> String {
        if let Some(new_tz) = timezone {
            TIMEZONE.with(|tz| {
                *tz.borrow_mut() = new_tz;
            });
        }
        
        let mut html = String::new();

        for node in &self.content {
            html.push_str(&node.to_html());
        }

        html
    }
}
