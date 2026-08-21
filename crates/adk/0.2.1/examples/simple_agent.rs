use adk::agent::AgentBuilder;
use adk::openai::OpenAI;
use adk::prelude::*;
use adk::{AgentError, Tool, ToolResult};
use async_trait::async_trait;
use std::sync::Arc;

// Define a simple calculator tool
struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "A simple calculator that can perform basic arithmetic operations"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "a": {
                    "type": "number"
                },
                "b": {
                    "type": "number"
                }
            },
            "required": ["operation", "a", "b"]
        })
    }

    async fn execute(
        &self,
        _context: &mut RunContext,
        params: &str,
    ) -> Result<ToolResult, AgentError> {
        let params: serde_json::Value = serde_json::from_str(params)?;

        let operation = params["operation"]
            .as_str()
            .ok_or_else(|| AgentError::InvalidInput("Missing operation".into()))?;
        let a = params["a"]
            .as_f64()
            .ok_or_else(|| AgentError::InvalidInput("Invalid first number".into()))?;
        let b = params["b"]
            .as_f64()
            .ok_or_else(|| AgentError::InvalidInput("Invalid second number".into()))?;

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(AgentError::InvalidInput("Division by zero".into()));
                }
                a / b
            }
            _ => return Err(AgentError::InvalidInput("Invalid operation".into())),
        };

        Ok(ToolResult {
            tool_name: self.name().to_string(),
            output: result.to_string(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    // Initialize the OpenAI model
    let model = Arc::new(OpenAI::new(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
        "gpt-4",
    ));

    // Create the calculator tool
    let calculator = Arc::new(CalculatorTool);

    // Create an agent using the builder pattern
    let agent = AgentBuilder::new("math_agent")
        .instructions("You are a helpful math assistant. Use the calculator tool to perform calculations when needed.")
        .model(model)
        .add_tool(calculator)
        .build()?;

    // Run the agent with a math problem
    let result = agent
        .run(
            "What is 123 + 456? Please use the calculator tool to compute this.",
            Context::new(),
        )
        .await?;

    println!("Agent response: {}", result);

    Ok(())
}
