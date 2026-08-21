use serde::{Deserialize, Serialize};

use crate::{
    marks::mark::{Mark, MarkVecToHtml as _}, 
    ToHtml
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Text {
    pub text: String,
    pub marks: Option<Vec<Mark>>,
}

impl Text {
    pub fn new(text: String, marks: Option<Vec<Mark>>) -> Self {
        Self { text, marks }
    }
}

impl ToHtml for Text {
    fn to_html(&self) -> String {
        let mut style = String::new();
        let mut tag = "span";
        let mut tag_a_attributes = String::new();

        if let Some(marks) = &self.marks {
            style = marks.get_styles();

            if let Some(Mark::Link(link)) = marks.iter().find(|m| matches!(m, Mark::Link(_))) {
                tag = "a";
                tag_a_attributes = link.html_a_tag_attributes();
            }
        }

        style.push_str("padding: 4px;");

        format!(r#"<{tag} {tag_a_attributes} style = "{style}">{}</{tag}>"#, self.text)
    }
}
