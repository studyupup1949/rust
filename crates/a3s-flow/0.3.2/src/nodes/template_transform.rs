//! Built-in `"template-transform"` node — renders a Jinja2 template using
//! upstream node outputs and global variables as context.
//!
//! Mirrors Dify's Template node.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "template": "Hello {{ user.name }}! Your order #{{ order_id }} is {{ fetch.body.status }}."
//! }
//! ```
//!
//! The template context contains:
//! - All global flow `variables` (accessible by key)
//! - All upstream node outputs (accessible by node ID)
//!
//! Upstream inputs take precedence over variables with the same key.
//!
//! Jinja2 syntax is supported via [minijinja](https://github.com/mitsuhiko/minijinja):
//! `{{ expr }}`, `{% if %}`, `{% for %}`, filters, etc.
//!
//! # Output schema
//!
//! ```json
//! { "output": "Hello Alice! Your order #42 is shipped." }
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// Template rendering node (Dify-compatible).
pub struct TemplateTransformNode;

#[async_trait]
impl Node for TemplateTransformNode {
    fn node_type(&self) -> &str {
        "template-transform"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let template_str = ctx.data["template"].as_str().ok_or_else(|| {
            FlowError::InvalidDefinition("template-transform: missing data.template".into())
        })?;

        // Build context: variables first, then inputs (inputs shadow variables).
        let mut context: std::collections::HashMap<String, Value> =
            ctx.variables.into_iter().collect();
        for (k, v) in ctx.inputs {
            context.insert(k, v);
        }

        let env = minijinja::Environment::new();
        let rendered = env
            .render_str(template_str, &context)
            .map_err(|e| FlowError::Internal(format!("template-transform: {e}")))?;

        Ok(json!({ "output": rendered }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx(
        inputs: HashMap<String, Value>,
        variables: HashMap<String, Value>,
        template: &str,
    ) -> ExecContext {
        ExecContext {
            data: json!({ "template": template }),
            inputs,
            variables,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn renders_simple_variable() {
        let node = TemplateTransformNode;
        let out = node
            .execute(ctx(
                HashMap::from([("user".into(), json!({ "name": "Alice" }))]),
                HashMap::new(),
                "Hello {{ user.name }}!",
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], "Hello Alice!");
    }

    #[tokio::test]
    async fn global_variables_available() {
        let node = TemplateTransformNode;
        let out = node
            .execute(ctx(
                HashMap::new(),
                HashMap::from([("env".into(), json!("production"))]),
                "env={{ env }}",
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], "env=production");
    }

    #[tokio::test]
    async fn inputs_shadow_variables() {
        let node = TemplateTransformNode;
        let out = node
            .execute(ctx(
                HashMap::from([("key".into(), json!("from_input"))]),
                HashMap::from([("key".into(), json!("from_var"))]),
                "{{ key }}",
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], "from_input");
    }

    #[tokio::test]
    async fn jinja2_if_block() {
        let node = TemplateTransformNode;
        let out = node
            .execute(ctx(
                HashMap::from([("fetch".into(), json!({ "ok": true }))]),
                HashMap::new(),
                "{% if fetch.ok %}ok{% else %}fail{% endif %}",
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], "ok");
    }

    #[tokio::test]
    async fn rejects_missing_template() {
        let node = TemplateTransformNode;
        let err = node
            .execute(ExecContext {
                data: json!({}),
                inputs: HashMap::new(),
                variables: HashMap::new(),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }
}
