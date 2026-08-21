//! `"answer"` node — returns a fixed or templated text answer.
//!
//! Mirrors Dify's Answer node.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "variables": ["query"],
//!   "answer": "The answer is: {{ query }}"
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `variables` | array | — | List of variable selectors available to the answer |
//! | `answer` | string | ✅ | Answer text — Jinja2 template rendered at execution time |
//!
//! # Output schema
//!
//! ```json
//! { "answer": "The answer is: Hello world!" }
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// Answer node — renders a templated text response.
pub struct AnswerNode;

#[async_trait]
impl Node for AnswerNode {
    fn node_type(&self) -> &str {
        "answer"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let answer_template = ctx
            .data
            .get("answer")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Build Jinja2 context: variables first, then inputs (inputs shadow variables)
        let mut context: HashMap<String, Value> = ctx.variables.into_iter().collect();
        for (k, v) in ctx.inputs {
            context.insert(k, v);
        }

        let env = minijinja::Environment::new();
        let rendered = env
            .render_str(answer_template, &context)
            .map_err(|e| FlowError::Internal(format!("answer: template error: {}", e)))?;

        Ok(json!({ "answer": rendered }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn renders_plain_answer() {
        let ctx = ExecContext {
            data: json!({ "answer": "Hello world" }),
            inputs: HashMap::new(),
            variables: HashMap::new(),
            ..Default::default()
        };
        let result = AnswerNode.execute(ctx).await.unwrap();
        assert_eq!(result["answer"], "Hello world");
    }

    #[tokio::test]
    async fn renders_template_with_variable() {
        let ctx = ExecContext {
            data: json!({ "answer": "Hello {{ name }}" }),
            inputs: HashMap::new(),
            variables: [("name".into(), json!("Alice"))].into(),
            ..Default::default()
        };
        let result = AnswerNode.execute(ctx).await.unwrap();
        assert_eq!(result["answer"], "Hello Alice");
    }
}
