//! Handler for the `transform_card` tool.

use adaptive_card_core::{CardVersion, Host, TransformTarget, transform_card};
use serde_json::Value;

pub fn handle(args: Value) -> Result<Value, String> {
    let card = args.get("card").cloned().ok_or("missing 'card'")?;
    let target = TransformTarget {
        version: args
            .get("target_version")
            .and_then(Value::as_str)
            .and_then(CardVersion::parse),
        host: args
            .get("target_host")
            .and_then(Value::as_str)
            .and_then(Host::from_str),
        strict: args.get("strict").and_then(Value::as_bool).unwrap_or(false),
    };
    transform_card(card, &target)
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .map_err(|e| crate::errors::to_tool_error_text(&e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn downgrades_version() {
        let args = json!({
            "card": {
                "type": "AdaptiveCard",
                "version": "1.6",
                "body": [{"type": "TextBlock", "text": "hi"}]
            },
            "target_version": "1.4"
        });
        let v = handle(args).unwrap();
        assert_eq!(v["card"]["version"], "1.4");
    }
}
