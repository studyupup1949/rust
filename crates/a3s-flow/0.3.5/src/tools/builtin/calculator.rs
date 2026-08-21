//! `"calculator"` built-in tool — evaluates mathematical expressions.

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::{FlowError, Result};
use crate::tools::tool::{Tool, ToolOutput};

/// Simple math expression evaluator using the `meval` crate.
///
/// Supports standard arithmetic: `+`, `-`, `*`, `/`, `^` (power), parentheses,
/// and common functions: `sin`, `cos`, `tan`, `log`, `ln`, `sqrt`, `abs`, `exp`.
///
/// # Example
///
/// | expression | result |
/// |------------|--------|
/// | `2 + 3 * 4` | 14 |
/// | `sqrt(16)` | 4 |
/// | `sin(pi/2)` | 1 |
pub struct CalculatorTool;

impl CalculatorTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn tool_name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate a mathematical expression and return the numeric result. Supports +, -, *, /, ^, sqrt, sin, cos, tan, log, ln, exp, abs, and parentheses."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate, e.g. '2 + 3 * 4' or 'sqrt(16)'"
                }
            },
            "required": ["expression"]
        })
    }

    async fn invoke(&self, args: Value) -> Result<ToolOutput> {
        let expression = args["expression"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidDefinition("calculator: expression is required".into()))?;

        let result = meval::eval_str(expression)
            .map_err(|e| FlowError::InvalidDefinition(format!("calculator: {}", e)))?;

        Ok(ToolOutput::ok(serde_json::to_string(&result).map_err(|e| {
            FlowError::Internal(format!("calculator: failed to serialize result: {}", e))
        })?))
    }
}

// Use meval for the actual evaluation
impl CalculatorTool {
    pub fn eval(&self, expression: &str) -> Result<f64> {
        meval::eval_str(expression)
            .map_err(|e| FlowError::InvalidDefinition(format!("calculator: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_basic() {
        let tool = CalculatorTool::new();
        assert!((tool.eval("2 + 3 * 4").unwrap() - 14.0).abs() < 1e-9);
        assert!((tool.eval("10.0 / 2.0").unwrap() - 5.0).abs() < 1e-9);
        assert!((tool.eval("2.0 ^ 8.0").unwrap() - 256.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn calculator_rejects_missing_expression() {
        let tool = CalculatorTool::new();
        let result = tool.invoke(json!({})).await;
        assert!(result.is_err());
    }
}
