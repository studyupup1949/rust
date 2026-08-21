// adminx-core/src/export.rs
//
// Serialize a set of rows for download as JSON or CSV.

use serde_json::Value;

/// Maximum rows included in an export (guards memory / response size).
pub const EXPORT_CAP: u64 = 10_000;

/// Render rows to CSV with the given column order. Values are stringified;
/// strings/numbers/bools render plainly, other JSON as compact text.
pub fn rows_to_csv(headers: &[String], rows: &[Value]) -> String {
    let mut out = String::new();
    out.push_str(
        &headers
            .iter()
            .map(|h| csv_escape(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');

    for row in rows {
        let line = headers
            .iter()
            .map(|h| csv_escape(&cell_value(row.get(h))))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&line);
        out.push('\n');
    }
    out
}

fn cell_value(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(['"', ',', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn csv_has_header_and_escapes() {
        let headers = vec!["id".to_string(), "name".to_string()];
        let rows = vec![
            json!({"id": 1, "name": "Ada"}),
            json!({"id": 2, "name": "a, b \"c\""}),
        ];
        let csv = rows_to_csv(&headers, &rows);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "id,name");
        assert_eq!(lines[1], "1,Ada");
        assert_eq!(lines[2], "2,\"a, b \"\"c\"\"\"");
    }
}
