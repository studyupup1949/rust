//! `refine-for-host` prompt renderer.

use serde_json::Value;

#[must_use]
pub fn render(args: &Value) -> String {
    let host = args
        .get("target_host")
        .and_then(Value::as_str)
        .unwrap_or("teams");
    let card_json = args
        .get("card")
        .map_or_else(|| "<missing>".to_string(), ToString::to_string);
    format!(
        "Refine this card to work on '{host}'.\n\
         Steps:\n\
         1. Call validate_card with host='{host}' to see incompatibilities\n\
         2. Call transform_card with target_host='{host}' to adapt\n\
         3. Call validate_card again to confirm\n\
         4. Return the refined card plus a list of changes\n\n\
         Card: {card_json}"
    )
}
