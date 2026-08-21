//! Handler for the `suggest_layout` tool.

use super::ToolCtx;
use serde_json::{Value, json};

pub fn handle(ctx: &ToolCtx, args: Value) -> Result<Value, String> {
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .ok_or("missing 'query'")?;
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .map(|u| u as usize)
        .unwrap_or(5);
    let results = ctx.kb.suggest(query, limit);
    Ok(json!(results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use adaptive_card_core::KnowledgeBase;
    use std::sync::LazyLock;

    static KB: LazyLock<KnowledgeBase> = LazyLock::new(KnowledgeBase::default);

    #[test]
    fn empty_kb_returns_empty_results() {
        let ctx = ToolCtx { kb: &KB };
        let v = handle(&ctx, json!({ "query": "any" })).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 0);
    }

    #[test]
    fn missing_query_is_error() {
        let ctx = ToolCtx { kb: &KB };
        let err = handle(&ctx, json!({})).unwrap_err();
        assert!(err.contains("query"));
    }
}
