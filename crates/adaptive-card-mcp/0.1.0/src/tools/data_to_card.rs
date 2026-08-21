//! Handler for the `data_to_card` tool.

use adaptive_card_core::{DataToCardOpts, Host, Presentation, data_to_card};
use serde_json::Value;

pub fn handle(args: Value) -> Result<Value, String> {
    let data = args.get("data").ok_or("missing 'data'")?;
    let host = args
        .get("host")
        .and_then(Value::as_str)
        .and_then(Host::from_str)
        .unwrap_or(Host::Generic);
    let presentation = args
        .get("presentation")
        .and_then(Value::as_str)
        .and_then(|s| match s {
            "table" => Some(Presentation::Table),
            "factset" | "fact_set" => Some(Presentation::FactSet),
            "list" => Some(Presentation::List),
            "chart" => Some(Presentation::Chart),
            "auto" => Some(Presentation::Auto),
            _ => None,
        });
    let title = args.get("title").and_then(Value::as_str).map(String::from);
    let opts = DataToCardOpts {
        title,
        presentation,
        host,
    };
    data_to_card(data, &opts).map_err(|e| crate::errors::to_tool_error_text(&e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn converts_list_to_card() {
        let args = json!({
            "data": [
                {"name": "a", "value": 1},
                {"name": "b", "value": 2}
            ],
            "host": "teams"
        });
        let v = handle(args).unwrap();
        assert_eq!(v["type"], "AdaptiveCard");
    }
}
