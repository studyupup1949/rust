//! `templatize-card` prompt renderer.

use serde_json::Value;

#[must_use]
pub fn render(args: &Value) -> String {
    let card_json = args
        .get("card")
        .map_or_else(|| "<missing>".to_string(), ToString::to_string);
    format!(
        "Convert this static card into a reusable template.\n\
         Steps:\n\
         1. Call template_card with the card\n\
         2. Call validate_card on the resulting template\n\
         3. Return the template, sample data, and list of bindings\n\n\
         Card: {card_json}"
    )
}
