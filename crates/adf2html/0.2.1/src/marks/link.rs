use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    #[serde(rename = "attrs")]
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub href: String,
    pub collection: Option<String>,
    pub id: Option<String>,
    pub occurrence_key: Option<String>,
    pub title: Option<String>,
}

impl Link {
    pub(crate) fn html_a_tag_attributes(&self) -> String {
        let mut html = format!(r#"href="{}""#, self.attributes.href);

        if let Some(id) = &self.attributes.id {
            html.push_str(&format!(r#" id="{id}""#));
        }

        if let Some(title) = &self.attributes.title {
            html.push_str(&format!(r#" title="{title}""#));
        }

        html
    }
}