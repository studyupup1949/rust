
use serde_json::{from_str, json, from_value, Value};
use crate::fs::temp::{TempFs};
use crate::utils::{Website};
use std::io::{Write};
use std::str;

pub(crate) fn results_to_string(file_manager: TempFs) -> String {
    let file_results: String = file_manager.read_results();
    let v: Value = from_str(&file_results).unwrap();

    v.to_string()
}

pub(crate) fn format_body(file_manager: TempFs) -> Value {
    let file_results: String = file_manager.read_results();
    let v: Value = from_str(&file_results).unwrap();
    let w = &v["website"];
    let website: Website = from_value(w.to_owned()).unwrap();
    let website_url = &website.url;
    let issues_length = website.issue.len();

    let seperator = if issues_length == 1 {
        ""
    } else {
        "s"
    }.to_string();

    let mut w = Vec::new();
    writeln!(&mut w).unwrap();
    writeln!(&mut w, "# {} issue{} found for {}", &issues_length, seperator, &website_url).unwrap();
    writeln!(&mut w, "<details>").unwrap();
    writeln!(&mut w, "<summary>").unwrap();
    writeln!(&mut w, "Details").unwrap();
    writeln!(&mut w, "</summary>").unwrap();

    for issue in website.issue {
        writeln!(&mut w, "<strong>{}</strong> <em>", issue.issue_type).unwrap();
        writeln!(&mut w, "{}", issue.code).unwrap();
        writeln!(&mut w, "</em>").unwrap();
        writeln!(&mut w, "").unwrap();
        writeln!(&mut w, "```html").unwrap();
        writeln!(&mut w, "{}", issue.context).unwrap();
        writeln!(&mut w, "```").unwrap();
        writeln!(&mut w, "").unwrap();
        writeln!(&mut w, "{}", issue.message).unwrap();
        writeln!(&mut w).unwrap();
        writeln!(&mut w, "---").unwrap();
    }
    
    writeln!(&mut w, "</details>").unwrap();
    writeln!(&mut w, "").unwrap();
    writeln!(&mut w, "---").unwrap();

    let body = str::from_utf8(&w).unwrap();

    json!({
        "body": body,
    })
}