//! Handler for the `validate_card` tool.

use adaptive_card_core::{Host, validate_card};
use serde_json::Value;

pub fn handle(args: Value) -> Result<Value, String> {
    let card = args.get("card").ok_or("missing 'card'")?;
    let host = args
        .get("host")
        .and_then(Value::as_str)
        .and_then(Host::from_str);
    let report = validate_card(card, host);
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_simple_card() {
        let args = json!({
            "card": {
                "type": "AdaptiveCard",
                "version": "1.6",
                "speak": "hi",
                "body": [{"type": "TextBlock", "text": "Hi", "wrap": true}]
            },
            "host": "teams"
        });
        let result = handle(args).unwrap();
        assert_eq!(result["valid"], true);
    }

    #[test]
    fn missing_card_returns_error() {
        let err = handle(json!({})).unwrap_err();
        assert!(err.contains("card"));
    }
}
