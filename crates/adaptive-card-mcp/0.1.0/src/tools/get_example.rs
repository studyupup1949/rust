//! Handler for the `get_example` tool.

use super::ToolCtx;
use serde_json::Value;

pub fn handle(ctx: &ToolCtx, args: Value) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(Value::as_str)
        .ok_or("missing 'id'")?;
    let entry = ctx
        .kb
        .by_id(id)
        .ok_or_else(|| format!("knowledge entry not found: {id}"))?;
    serde_json::to_value(entry).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adaptive_card_core::KnowledgeBase;
    use serde_json::json;
    use std::sync::LazyLock;

    static KB: LazyLock<KnowledgeBase> = LazyLock::new(KnowledgeBase::default);

    #[test]
    fn missing_id_reports_error() {
        let ctx = ToolCtx { kb: &KB };
        let err = handle(&ctx, json!({})).unwrap_err();
        assert!(err.contains("id"));
    }

    #[test]
    fn unknown_id_reports_error() {
        let ctx = ToolCtx { kb: &KB };
        let err = handle(&ctx, json!({ "id": "nope" })).unwrap_err();
        assert!(err.contains("not found"));
    }
}
