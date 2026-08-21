use serde::{Deserialize, Serialize};

use crate::ToHtml;use super::media::Media;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaGroup {
    pub content: Vec<Media>,
}

impl ToHtml for MediaGroup {
    fn to_html(&self) -> String {
        self.content
            .iter()
            .map(|m| format!(r#"<p style = "padding: 4px;">{}</p>"#, m.to_html()))
            .collect()
    }
}
