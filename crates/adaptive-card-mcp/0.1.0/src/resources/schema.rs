//! `ac://schema/v1.6` — the Microsoft Adaptive Cards v1.6 JSON Schema.

use serde_json::Value;

#[must_use]
pub fn body() -> Value {
    adaptive_card_core::schema::v1_6_schema().clone()
}
