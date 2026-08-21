use adk::AgentError;
use adk::agent::AgentBuilder;
use adk::openai::OpenAI;
use adk::prelude::*;
use adk::tool_fn;
use std::sync::Arc;

// Define a calculator tool using the tool_fn macro
#[tool_fn(
    name = "calculator",
    description = "A simple calculator that can perform basic arithmetic operations(add, subtract, multiply, divide)"
)]
fn calculator(_context: &mut RunContext, a: f64, b: f64, operation: String) -> String {
    // Perform the calculation based on the operation
    let result = match operation.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return "Error: Division by zero".to_string();
            }
            a / b
        }
        _ => return format!("Error: Invalid operation '{}'", operation),
    };

    // Return the result as a string
    result.to_string()
}

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    // Initialize the OpenAI model
    let model = Arc::new(OpenAI::new(
        std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
        "gpt-4",
    ));

    // Create the calculator tool using the generated function from the macro
    let calculator = Arc::new(calculator_tool());

    // Create an agent using the builder pattern
    let agent = AgentBuilder::new("math_agent")
        .instructions("You are a helpful math assistant. Use the calculator tool to perform calculations when needed.")
        .model(model)
        .add_tool(calculator)
        .build()?;

    // Run the agent with a math problem
    let result = agent
        .run(
            "What is 15.7 * 9.2? Please use the calculator tool to compute this.",
            Context::new(),
        )
        .await?;

    println!("Agent response: {}", result);

    Ok(())
}
