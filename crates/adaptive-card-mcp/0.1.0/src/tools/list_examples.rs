//! Handler for the `list_examples` tool.

use super::ToolCtx;
use serde_json::{Value, json};

pub fn handle(ctx: &ToolCtx, args: Value) -> Result<Value, String> {
    let category = args.get("category").and_then(Value::as_str);
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .map(|u| u as usize)
        .unwrap_or(20);
    let entries: Vec<Value> = ctx
        .kb
        .all()
        .iter()
        .filter(|e| category.is_none_or(|c| e.category == c))
        .take(limit)
        .map(|e| {
            json!({
                "id": e.id,
                "title": e.title,
                "category": e.category,
                "complexity": format!("{:?}", e.complexity).to_lowercase(),
                "tags": e.tags,
            })
        })
        .collect();
    Ok(Value::Array(entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use adaptive_card_core::KnowledgeBase;
    use std::sync::LazyLock;

    static KB: LazyLock<KnowledgeBase> = LazyLock::new(KnowledgeBase::default);

    #[test]
    fn returns_empty_array_when_kb_empty() {
        let ctx = ToolCtx { kb: &KB };
        let v = handle(&ctx, json!({})).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 0);
    }
}
