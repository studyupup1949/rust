//! Handler for the `analyze_card` tool.

use adaptive_card_core::analyze_card;
use serde_json::Value;

pub fn handle(args: Value) -> Result<Value, String> {
    let card = args.get("card").ok_or("missing 'card'")?;
    serde_json::to_value(analyze_card(card)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn counts_elements() {
        let args = json!({
            "card": {
                "type": "AdaptiveCard",
                "version": "1.6",
                "body": [
                    {"type": "TextBlock", "text": "a"},
                    {"type": "TextBlock", "text": "b"}
                ]
            }
        });
        let v = handle(args).unwrap();
        assert_eq!(v["element_count"], 2);
    }
}
