use async_trait::async_trait;

use crate::error::AgentResult;
use crate::types::RunContext;

// Re-export the procedural macro
pub use adk_macros::tool_fn;

/// Represents the result of a tool execution
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// The name of the tool that was executed
    pub tool_name: String,
    /// The output of the tool execution
    pub output: String,
}

/// Trait for tools that can be used by agents
#[async_trait]
pub trait Tool: Send + Sync {
    /// The name of the tool
    fn name(&self) -> &str;

    /// A description of what the tool does
    fn description(&self) -> &str;

    /// The JSON schema for the tool's parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Execute the tool with the given parameters
    async fn execute(&self, context: &mut RunContext, params: &str) -> AgentResult<ToolResult>;
}

type FunctionToolFn = Box<dyn Fn(&mut RunContext, &str) -> AgentResult<ToolResult> + Send + Sync>;

/// A function-based tool implementation
pub struct FunctionTool {
    name: String,
    description: String,
    parameters_schema: serde_json::Value,
    function: FunctionToolFn,
}

impl FunctionTool {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters_schema: serde_json::Value,
        function: FunctionToolFn,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters_schema,
            function,
        }
    }
}

#[async_trait]
impl Tool for FunctionTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.parameters_schema.clone()
    }

    async fn execute(&self, context: &mut RunContext, params: &str) -> AgentResult<ToolResult> {
        (self.function)(context, params)
    }
}

/// A macro to create a function tool with automatic parameter schema generation
#[macro_export]
macro_rules! function_tool {
    // Original simple case (no parameters)
    ($name:expr, $description:expr, $function:expr) => {
        $crate::tool::FunctionTool::new(
            $name,
            $description,
            serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            Box::new($function),
        )
    };

    // New case that accepts custom schema
    ($name:expr, $description:expr, $schema:expr, $function:expr) => {
        $crate::tool::FunctionTool::new(
            $name,
            $description,
            $schema,
            Box::new($function),
        )
    };
}
