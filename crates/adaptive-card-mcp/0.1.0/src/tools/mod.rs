//! MCP tool handler functions.
//!
//! Each handler takes arguments as a JSON [`serde_json::Value`] and returns
//! either the tool output (also a [`serde_json::Value`]) or an error string
//! suitable for display to the calling LLM.

#![allow(
    dead_code,
    reason = "handlers are wired into the rmcp server in Task 35"
)]

pub mod accessibility;
pub mod analyze;
pub mod data_to_card;
pub mod get_example;
pub mod list_examples;
pub mod optimize;
pub mod schemas;
pub mod suggest_layout;
pub mod template;
pub mod transform;
pub mod validate;

use adaptive_card_core::KnowledgeBase;
use serde_json::Value;

/// Context shared by every tool invocation (currently just the knowledge base).
#[derive(Copy, Clone)]
pub struct ToolCtx {
    pub kb: &'static KnowledgeBase,
}

/// List of every tool exposed by this server, in the order clients should
/// see them in `tools/list`.
pub const TOOL_NAMES: &[&str] = &[
    "validate_card",
    "analyze_card",
    "check_accessibility",
    "optimize_card",
    "transform_card",
    "template_card",
    "data_to_card",
    "list_examples",
    "get_example",
    "suggest_layout",
];

/// Dispatch a tool call by name.
///
/// Returns `Ok(value)` on success or `Err(message)` if the tool rejected the
/// arguments or the core library returned an error.
pub fn dispatch(ctx: &ToolCtx, name: &str, args: Value) -> Result<Value, String> {
    match name {
        "validate_card" => validate::handle(args),
        "analyze_card" => analyze::handle(args),
        "check_accessibility" => accessibility::handle(args),
        "optimize_card" => optimize::handle(args),
        "transform_card" => transform::handle(args),
        "template_card" => template::handle(args),
        "data_to_card" => data_to_card::handle(args),
        "list_examples" => list_examples::handle(ctx, args),
        "get_example" => get_example::handle(ctx, args),
        "suggest_layout" => suggest_layout::handle(ctx, args),
        _ => Err(format!("unknown tool: {name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adaptive_card_core::KnowledgeBase;
    use serde_json::json;
    use std::sync::LazyLock;

    static KB: LazyLock<KnowledgeBase> = LazyLock::new(KnowledgeBase::default);

    fn ctx() -> ToolCtx {
        ToolCtx { kb: &KB }
    }

    #[test]
    fn unknown_tool_yields_error() {
        let err = dispatch(&ctx(), "nope", json!({})).unwrap_err();
        assert!(err.contains("unknown tool"));
    }

    #[test]
    fn all_ten_tools_are_named() {
        assert_eq!(TOOL_NAMES.len(), 10);
    }

    #[test]
    fn every_named_tool_resolves() {
        // Call each tool with an empty object; each handler either runs or
        // returns a predictable missing-argument error — but none should
        // hit the "unknown tool" branch.
        for name in TOOL_NAMES {
            let res = dispatch(&ctx(), name, json!({}));
            if let Err(msg) = &res {
                assert!(
                    !msg.contains("unknown tool"),
                    "tool '{name}' should be registered"
                );
            }
        }
    }
}
