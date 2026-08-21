//! Handler for the `optimize_card` tool.

use adaptive_card_core::{Host, OptimizeOpts, optimize_card};
use serde_json::Value;

pub fn handle(args: Value) -> Result<Value, String> {
    let card = args.get("card").cloned().ok_or("missing 'card'")?;
    let opts = OptimizeOpts {
        accessibility: args
            .get("accessibility")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        performance: args
            .get("performance")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        modernize: args
            .get("modernize")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        target_host: args
            .get("target_host")
            .and_then(Value::as_str)
            .and_then(Host::from_str),
    };
    serde_json::to_value(optimize_card(card, &opts)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn optimizes_a_simple_card() {
        let args = json!({
            "card": {
                "type": "AdaptiveCard",
                "version": "1.6",
                "body": [{"type": "TextBlock", "text": "hi"}]
            },
            "accessibility": true
        });
        let v = handle(args).unwrap();
        assert!(v["card"].is_object());
    }
}
