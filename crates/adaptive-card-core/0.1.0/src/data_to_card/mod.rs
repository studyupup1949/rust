//! Generate a minimal Adaptive Card from arbitrary data.

pub mod shape;

use crate::error::{Error, Result};
use crate::types::{DataToCardOpts, Presentation};
use serde_json::{Value, json};

/// Generate an Adaptive Card from input data.
pub fn data_to_card(data: &Value, opts: &DataToCardOpts) -> Result<Value> {
    let presentation = match opts.presentation.unwrap_or(Presentation::Auto) {
        Presentation::Auto => shape::detect(data),
        p => p,
    };

    let title_block = opts.title.as_ref().map(|t| {
        json!({
            "type": "TextBlock",
            "text": t,
            "size": "Large",
            "weight": "Bolder",
            "wrap": true
        })
    });

    let body_element = match presentation {
        Presentation::FactSet => fact_set_from_object(data)?,
        Presentation::Table => table_from_array(data)?,
        Presentation::List => list_from_value(data),
        Presentation::Chart => {
            return Err(Error::Internal(
                "Chart presentation not yet supported".to_string(),
            ));
        }
        Presentation::Auto => unreachable!(),
    };

    let mut body: Vec<Value> = Vec::new();
    if let Some(t) = title_block {
        body.push(t);
    }
    body.push(body_element);

    Ok(json!({
        "type": "AdaptiveCard",
        "version": opts.host.max_version().as_str(),
        "body": body,
        "speak": opts.title.as_deref().unwrap_or("Data card")
    }))
}

fn fact_set_from_object(data: &Value) -> Result<Value> {
    let obj = data.as_object().ok_or(Error::UnrecognizedDataShape)?;
    let facts: Vec<Value> = obj
        .iter()
        .map(|(k, v)| {
            json!({
                "title": k,
                "value": format_value(v)
            })
        })
        .collect();
    Ok(json!({ "type": "FactSet", "facts": facts }))
}

fn table_from_array(data: &Value) -> Result<Value> {
    let arr = data.as_array().ok_or(Error::UnrecognizedDataShape)?;
    let first = arr
        .first()
        .and_then(Value::as_object)
        .ok_or(Error::UnrecognizedDataShape)?;
    let headers: Vec<String> = first.keys().cloned().collect();

    let mut rows: Vec<Value> = Vec::new();
    // Header row
    rows.push(json!({
        "type": "TableRow",
        "cells": headers.iter().map(|h| json!({
            "type": "TableCell",
            "items": [{ "type": "TextBlock", "text": h, "weight": "Bolder" }]
        })).collect::<Vec<_>>()
    }));
    // Data rows
    for row in arr {
        let obj = row.as_object().ok_or(Error::UnrecognizedDataShape)?;
        let cells: Vec<Value> = headers
            .iter()
            .map(|h| {
                json!({
                    "type": "TableCell",
                    "items": [{
                        "type": "TextBlock",
                        "text": format_value(obj.get(h).unwrap_or(&Value::Null))
                    }]
                })
            })
            .collect();
        rows.push(json!({ "type": "TableRow", "cells": cells }));
    }
    let columns: Vec<Value> = (0..headers.len()).map(|_| json!({ "width": 1 })).collect();
    Ok(json!({
        "type": "Table",
        "columns": columns,
        "rows": rows
    }))
}

fn list_from_value(data: &Value) -> Value {
    let items: Vec<Value> = match data {
        Value::Array(arr) => arr
            .iter()
            .map(|v| {
                json!({
                    "type": "TextBlock",
                    "text": format!("• {}", format_value(v)),
                    "wrap": true
                })
            })
            .collect(),
        _ => vec![json!({
            "type": "TextBlock",
            "text": format_value(data),
            "wrap": true
        })],
    };
    json!({ "type": "Container", "items": items })
}

fn format_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "—".to_string(),
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Host;
    use serde_json::json;

    #[test]
    fn single_object_produces_factset() {
        let data = json!({ "name": "Alice", "age": 30 });
        let card = data_to_card(
            &data,
            &DataToCardOpts {
                title: Some("Profile".to_string()),
                presentation: None,
                host: Host::Teams,
            },
        )
        .unwrap();
        assert_eq!(card["body"][0]["text"], "Profile");
        assert_eq!(card["body"][1]["type"], "FactSet");
    }

    #[test]
    fn array_of_objects_produces_table() {
        let data = json!([
            { "region": "APAC", "revenue": 1000 },
            { "region": "EMEA", "revenue": 2000 }
        ]);
        let card = data_to_card(
            &data,
            &DataToCardOpts {
                title: None,
                presentation: None,
                host: Host::Teams,
            },
        )
        .unwrap();
        assert_eq!(card["body"][0]["type"], "Table");
    }

    #[test]
    fn outlook_target_caps_version() {
        let data = json!({ "x": 1 });
        let card = data_to_card(
            &data,
            &DataToCardOpts {
                title: None,
                presentation: None,
                host: Host::Outlook,
            },
        )
        .unwrap();
        assert_eq!(card["version"], "1.4");
    }
}
