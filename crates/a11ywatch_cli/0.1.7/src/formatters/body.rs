
use serde_json::{from_str, json, from_value, Value};
use crate::fs::temp::{TempFs};
use crate::utils::{Website};
use std::io::{Write};
use std::str;

/// read json results to string
pub(crate) fn results_to_string(file_manager: &TempFs) -> String {
    let file_results: String = file_manager.read_results();
    let v: Value = from_str(&file_results).unwrap();

    v.to_string()
}

/// read github html format message
pub(crate) fn results_to_string_github(file_manager: &TempFs) -> String {
    let file_results: String = file_manager.read_results_github();
    let v: Value = from_str(&file_results).unwrap();

    v.to_string()
}

pub(crate) fn results_to_value(file_manager: &TempFs) -> Value {
    let file_results: String = file_manager.read_results();
    let v: Value = from_str(&file_results).unwrap();

    v
}

// the amount of issues for the page from previus scan
pub(crate) fn results_issues_count(file_manager: &TempFs) -> usize {
    let json_data = results_to_value(&file_manager);
    let data = &json_data["data"];
    let mut count: usize = 0;
    
    if data.is_array() {
        let pages = data.as_array().unwrap();
    
        for page in pages {
            let issues = &page["issues"];
            if issues.is_array() {
                let issues = issues.as_array().unwrap();
                count += issues.len();
            }
        }
    }
    if data.is_object() {
        let issues = &data["issues"];
    
        if issues.is_array() {
            let issues = issues.as_array().unwrap();
            count += issues.len();
        }
    }

    count
}

// format the body for sending to github. Handles multi page and single page reports.
pub(crate) fn format_body(file_manager: &TempFs) -> Value {
    let v: Value = results_to_value(file_manager);
    let data = &v["data"];

    if data.is_array() {
        let pages = data.as_array().unwrap();
        let mut w = Vec::new();
        let count = results_issues_count(file_manager);
        
        let seperator = if count == 1 {
            ""
        } else {
            "s"
        }.to_string();

        let first_page: Website = from_value(pages[0].clone()).unwrap();

        // get the top level domain for the pages
        let domain = &first_page.domain;

        writeln!(&mut w).unwrap();
        writeln!(&mut w, "# {} issue{} found for {}", &count, seperator, &domain).unwrap();
        writeln!(&mut w, "<details>").unwrap();
        writeln!(&mut w, "<summary>").unwrap();
        writeln!(&mut w, "Details").unwrap();
        writeln!(&mut w, "</summary>").unwrap();
        writeln!(&mut w, "").unwrap();    
        writeln!(&mut w, "").unwrap();

        for page in pages {
            let website: Website = from_value(page.to_owned()).unwrap();
            let website_url = &website.url;
            let issues = website.issues.unwrap_or_default();

            let issues_length = issues.len();
        
            let seperator = if issues_length == 1 {
                ""
            } else {
                "s"
            }.to_string();
        
            writeln!(&mut w).unwrap();
            writeln!(&mut w, "<details>").unwrap();
            writeln!(&mut w, "<summary>").unwrap();
            writeln!(&mut w, "#{} - {} issue{}", &website_url, &issues_length, seperator).unwrap();
            writeln!(&mut w, "</summary>").unwrap();
            writeln!(&mut w, "").unwrap();
            writeln!(&mut w, "").unwrap();

            for issue in issues {
                writeln!(&mut w, "<strong>{}</strong> <em>{}</em>", issue.issue_type.to_uppercase(), issue.code).unwrap();
                writeln!(&mut w, "").unwrap();
                writeln!(&mut w, "```html").unwrap();
                writeln!(&mut w, "{}", issue.context.trim_end()).unwrap();
                writeln!(&mut w, "```").unwrap();
                writeln!(&mut w, "").unwrap();
                writeln!(&mut w, "{}", issue.message).unwrap();
                writeln!(&mut w).unwrap();
                writeln!(&mut w, "---").unwrap();
                writeln!(&mut w).unwrap();
            }
            
            writeln!(&mut w, "</details>").unwrap();
            writeln!(&mut w, "").unwrap();
            writeln!(&mut w, "---").unwrap();
            writeln!(&mut w, "").unwrap();
        }

        writeln!(&mut w, "</details>").unwrap();
        writeln!(&mut w, "").unwrap();

        write!(&mut w, "[📝 docs](https://docs.a11ywatch.com) | ").unwrap();
        write!(&mut w, "[:octocat: repo](https://github.com/A11yWatch/a11ywatch) | ").unwrap();
        write!(&mut w, "[🙋🏽‍♀️ issues](https://github.com/A11yWatch/a11ywatch/issues) | ").unwrap();
        write!(&mut w, "[🏪 marketplace](https://github.com/marketplace/actions/a11ywatch-the-accessibility-suite) | ").unwrap();
        write!(&mut w, "[A11yWatch](https://a11ywatch.com)").unwrap();
    
        writeln!(&mut w, "").unwrap();
        writeln!(&mut w, "").unwrap();

        let body = str::from_utf8(&w).unwrap();
        let json = json!({
            "body": body,
        });

        file_manager.save_github_results(&json).unwrap_or_default();

        json
    } else {
        let website: Website = from_value(data.to_owned()).unwrap();
        let domain = &website.domain;
        let issues = website.issues.unwrap_or_default();

        let issues_length = issues.len();
    
        let seperator = if issues_length == 1 {
            ""
        } else {
            "s"
        }.to_string();
    
        let mut w = Vec::new();
        writeln!(&mut w).unwrap();
        writeln!(&mut w, "# {} issue{} found for {}", &issues_length, seperator, &domain).unwrap();
        writeln!(&mut w, "<details>").unwrap();
        writeln!(&mut w, "<summary>").unwrap();
        writeln!(&mut w, "Details").unwrap();
        writeln!(&mut w, "</summary>").unwrap();
        writeln!(&mut w, "").unwrap();
    
        for issue in issues {
            writeln!(&mut w, "<strong>{}</strong> <em>{}</em>", issue.issue_type.to_uppercase(), issue.code).unwrap();
            writeln!(&mut w, "").unwrap();
            writeln!(&mut w, "```html").unwrap();
            writeln!(&mut w, "{}", issue.context.trim_end()).unwrap();
            writeln!(&mut w, "```").unwrap();
            writeln!(&mut w, "").unwrap();
            writeln!(&mut w, "{}", issue.message).unwrap();
            writeln!(&mut w).unwrap();
            writeln!(&mut w, "---").unwrap();
            writeln!(&mut w).unwrap();
        }
        
        writeln!(&mut w, "</details>").unwrap();
        writeln!(&mut w, "").unwrap();
        writeln!(&mut w, "---").unwrap();
        writeln!(&mut w, "").unwrap();
    
        write!(&mut w, "[📝 docs](https://docs.a11ywatch.com) | ").unwrap();
        write!(&mut w, "[:octocat: repo](https://github.com/A11yWatch/a11ywatch) | ").unwrap();
        write!(&mut w, "[🙋🏽‍♀️ issues](https://github.com/A11yWatch/a11ywatch/issues) | ").unwrap();
        write!(&mut w, "[🏪 marketplace](https://github.com/marketplace/actions/a11ywatch-the-accessibility-suite) | ").unwrap();
        write!(&mut w, "[A11yWatch](https://a11ywatch.com)").unwrap();
    
        writeln!(&mut w, "").unwrap();
        writeln!(&mut w, "").unwrap();
    
        let body = str::from_utf8(&w).unwrap();

        
        json!({
            "body": body,
        })
    }

}