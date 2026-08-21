use serde::{Deserialize, Serialize};

use super::table_cell::TableCell;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    pub content: Vec<Content>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Content {
    TableHeader(TableCell),
    TableCell(TableCell),
}

impl TableRow {
    pub(crate) fn to_html(&self, row_number: Option<u32>) -> String {
        let mut html = String::from("<tr>");

        if let Some(i) = row_number {
            html.push_str(&format!(r#"<th style = "background-color: #F1F2F4;">{i}</th>"#));
        }

        for cell in &self.content {
            match cell {
                Content::TableHeader(table_cell) => html.push_str(&table_cell.to_html(true)),
                Content::TableCell(table_cell) => html.push_str(&table_cell.to_html(false)),
            };
        }

        html.push_str("</tr>");

        html
    }
}
