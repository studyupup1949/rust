//! `ac://hosts[/{name}]` — host capability summaries.

use adaptive_card_core::Host;
use serde_json::{Value, json};

#[must_use]
pub fn all_body() -> Value {
    json!(
        Host::all()
            .iter()
            .map(|h| host_summary(*h))
            .collect::<Vec<_>>()
    )
}

#[must_use]
pub fn one_body(name: &str) -> Option<Value> {
    Host::from_str(name).map(host_summary)
}

fn host_summary(h: Host) -> Value {
    json!({
        "name": format!("{h:?}"),
        "max_version": h.max_version().as_str(),
        "max_actions": h.max_actions(),
        "supports_execute": h.supports_action("Action.Execute"),
        "supports_table": h.supports_element("Table"),
    })
}
