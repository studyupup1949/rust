//! Handler for the `check_accessibility` tool.

use adaptive_card_core::check_accessibility;
use serde_json::Value;

pub fn handle(args: Value) -> Result<Value, String> {
    let card = args.get("card").ok_or("missing 'card'")?;
    serde_json::to_value(check_accessibility(card)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reports_score() {
        let args = json!({
            "card": {
                "type": "AdaptiveCard",
                "version": "1.6",
                "speak": "hi",
                "body": [{"type": "TextBlock", "text": "Hi", "wrap": true}]
            }
        });
        let v = handle(args).unwrap();
        assert!(v["score"].is_number());
    }
}
