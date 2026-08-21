use crate::abi::AbiItem;
use crate::json_parser::JsonParser;

pub struct Converter;

impl Converter {
    pub fn parse_abi_content(content: &str) -> Result<Vec<AbiItem>, String> {
        let mut parser = JsonParser::new(content);
        parser.parse_abi()
    }

    pub fn convert_to_human_readable(abi: &[AbiItem]) -> Vec<String> {
        abi.iter()
            .filter(|item| !item.r#type.is_empty() && item.r#type != "unknown")
            .map(|item| item.to_string())
            .filter(|formatted| !formatted.is_empty() && !formatted.starts_with("{}"))
            .collect()
    }

    pub fn format_as_json_array(human_readable: &[String], pretty: bool) -> String {
        if pretty {
            format_json_pretty(human_readable)
        } else {
            format_json_compact(human_readable)
        }
    }
}

fn format_json_pretty(strings: &[String]) -> String {
    if strings.is_empty() {
        return "[]".to_string();
    }

    let mut result = String::from("[\n");
    for (i, s) in strings.iter().enumerate() {
        result.push_str("  \"");
        result.push_str(&escape_json_string(s));
        result.push('"');
        if i < strings.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }
    result.push(']');
    result
}

fn format_json_compact(strings: &[String]) -> String {
    let escaped: Vec<String> = strings
        .iter()
        .map(|s| format!("\"{}\"", escape_json_string(s)))
        .collect();
    format!("[{}]", escaped.join(","))
}

fn escape_json_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}
