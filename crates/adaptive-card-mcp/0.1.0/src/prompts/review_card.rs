//! `review-adaptive-card` prompt renderer.

use serde_json::Value;

#[must_use]
pub fn render(args: &Value) -> String {
    let host = args
        .get("host")
        .and_then(Value::as_str)
        .unwrap_or("generic");
    let card_json = args
        .get("card")
        .map_or_else(|| "<missing>".to_string(), ToString::to_string);
    format!(
        "Review this Adaptive Card and improve it for host '{host}'.\n\
         Steps:\n\
         1. Call validate_card with the card and host='{host}'\n\
         2. If there are schema errors, fix them\n\
         3. Call optimize_card with accessibility=true\n\
         4. Call validate_card again to confirm\n\
         5. Return a before/after summary: original a11y score, new a11y score, changes made\n\n\
         Card: {card_json}"
    )
}
