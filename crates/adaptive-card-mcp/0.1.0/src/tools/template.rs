//! Handler for the `template_card` tool.

use adaptive_card_core::template_card;
use serde_json::Value;

pub fn handle(args: Value) -> Result<Value, String> {
    let card = args.get("card").cloned().ok_or("missing 'card'")?;
    serde_json::to_value(template_card(card)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn produces_template_result() {
        let args = json!({
            "card": {
                "type": "AdaptiveCard",
                "version": "1.6",
                "body": [{"type": "TextBlock", "text": "Hello Alice"}]
            }
        });
        let v = handle(args).unwrap();
        assert!(v["template"].is_object());
        assert!(v["sample_data"].is_object());
    }
}
