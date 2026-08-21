use serde::{Deserialize, Serialize};

use crate::{
    marks::link::Link,
    ToHtml,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    #[serde(rename = "attrs")]
    pub attributes: Attributes,
    pub marks: Option<Vec<Mark>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub collection: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub occurrence_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    File,
    Link,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Mark {
    Border,
    Link(Link),
}

impl ToHtml for Media {
    fn to_html(&self) -> String {
        let mut html = format!("{}_{}", self.attributes.collection, self.attributes.id);
        
        if let Some(marks) = &self.marks {
            if let Some(Mark::Link(link)) = marks.iter().find(|m| matches!(m, Mark::Link(_))) {
                html = format!(r#"<a {} style = "padding: 4px;">{}</a>"#, link.html_a_tag_attributes(), html);
            }
        }

        html
    }
}
