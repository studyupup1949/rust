use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInline {
    #[serde(rename = "attrs")]
    pub attributes: Attributes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub id: Option<String>,
    pub collection: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub alt: Option<String>,
    pub url: Option<String>,
}

impl MediaInline {
    pub fn to_html(&self) -> String {
        let alt = self.attributes.alt.as_deref().unwrap_or("attachment");
        if let Some(url) = &self.attributes.url {
            use html_escape::{encode_quoted_attribute, encode_text};
            format!(
                r#"<a href="{}">{}</a>"#,
                encode_quoted_attribute(url),
                encode_text(alt)
            )
        } else {
            use html_escape::encode_text;
            format!(r#"<span class="media-inline">{}</span>"#, encode_text(alt))
        }
    }
}
