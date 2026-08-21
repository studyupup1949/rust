//! Example: Custom MCP (Model Context Protocol) Node
//!
//! This example demonstrates how to implement a custom node that integrates
//! with Model Context Protocol servers. MCP nodes are service-specific and
//! should be registered dynamically by users, not built into the engine.
//!
//! ## Key Concepts
//!
//! 1. **Implement the `Node` trait** — Define node type and execution logic
//! 2. **Register dynamically** — Add to `NodeRegistry` before creating engine
//! 3. **Access shared context** — Read/write global context via `ctx.context`
//! 4. **Use Jinja2 templates** — Render dynamic arguments from upstream data
//!
//! ## Running this example
//!
//! ```bash
//! cargo run --example custom_mcp_node
//! ```

use a3s_flow::{
    DagGraph, ExecContext, FlowEngine, FlowError, Node, NodeRegistry, Result as FlowResult,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// MCP node configuration (parsed from `data` field in flow definition)
#[derive(Debug, Deserialize)]
struct McpConfig {
    /// MCP server URL (e.g., "http://localhost:3000")
    server_url: String,
    /// Tool name to call on the MCP server
    tool_name: String,
    /// Transport protocol: "sse" or "http" (default: "sse")
    #[serde(default = "default_transport")]
    transport: String,
    /// Arguments to pass to the tool (supports Jinja2 templates)
    #[serde(default)]
    arguments: HashMap<String, Value>,
}

fn default_transport() -> String {
    "sse".to_string()
}

/// Custom MCP node implementation
pub struct McpNode;

#[async_trait]
impl Node for McpNode {
    fn node_type(&self) -> &str {
        "mcp"
    }

    async fn execute(&self, ctx: ExecContext) -> FlowResult<Value> {
        // Parse node configuration
        let config: McpConfig = serde_json::from_value(ctx.data.clone())
            .map_err(|e| FlowError::InvalidDefinition(format!("invalid mcp node config: {}", e)))?;

        // Example: Read from shared context (Dify-style global context)
        let conversation_id = {
            let context = ctx.context.read().unwrap();
            context
                .get("conversation_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string()
        };

        // Example: Render Jinja2 templates in arguments (simplified here)
        let rendered_args = self.render_arguments(&config.arguments, &ctx)?;

        // Simulate MCP tool call (in real implementation, use MCP client library)
        println!(
            "[MCP] Calling tool '{}' on server '{}' (transport: {})",
            config.tool_name, config.server_url, config.transport
        );
        println!("[MCP] Conversation ID: {}", conversation_id);
        println!("[MCP] Arguments: {:?}", rendered_args);

        // Simulate response
        let response = json!({
            "text": format!("Response from {} tool", config.tool_name),
            "content": [
                {
                    "type": "text",
                    "text": "This is a simulated MCP response"
                }
            ],
            "is_error": false
        });

        // Example: Write to shared context
        {
            let mut context = ctx.context.write().unwrap();
            context.insert(
                "last_mcp_call".to_string(),
                json!({
                    "tool": config.tool_name,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }),
            );
        }

        Ok(response)
    }
}

impl McpNode {
    /// Render Jinja2 templates in arguments (simplified version)
    fn render_arguments(
        &self,
        arguments: &HashMap<String, Value>,
        ctx: &ExecContext,
    ) -> FlowResult<HashMap<String, Value>> {
        let mut rendered = HashMap::new();

        for (key, value) in arguments {
            // In real implementation, use minijinja to render templates
            // For now, just pass through and substitute {{variables.x}} patterns
            let rendered_value = if let Some(s) = value.as_str() {
                if s.starts_with("{{") && s.ends_with("}}") {
                    // Extract variable name (simplified)
                    let var_name = s.trim_start_matches("{{").trim_end_matches("}}").trim();
                    if let Some(var_path) = var_name.strip_prefix("variables.") {
                        ctx.variables.get(var_path).cloned().unwrap_or(Value::Null)
                    } else {
                        value.clone()
                    }
                } else {
                    value.clone()
                }
            } else {
                value.clone()
            };

            rendered.insert(key.clone(), rendered_value);
        }

        Ok(rendered)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a custom registry with MCP node
    let mut registry = NodeRegistry::with_defaults();
    registry.register(Arc::new(McpNode));

    println!("✓ Registered custom MCP node");
    println!("  Available node types: {:?}\n", registry.list_types());

    // 2. Create a flow that uses the MCP node
    let flow_def = json!({
        "nodes": [
            {
                "id": "init",
                "type": "assign",
                "data": {
                    "assigns": {
                        "query": "What is the weather in San Francisco?"
                    }
                }
            },
            {
                "id": "mcp_call",
                "type": "mcp",
                "data": {
                    "server_url": "http://localhost:3000",
                    "tool_name": "get_weather",
                    "transport": "sse",
                    "arguments": {
                        "location": "{{variables.query}}"
                    }
                }
            },
            {
                "id": "end",
                "type": "end",
                "data": {
                    "outputs": {
                        "result": "/mcp_call/text"
                    }
                }
            }
        ],
        "edges": [
            { "source": "init", "target": "mcp_call" },
            { "source": "mcp_call", "target": "end" }
        ]
    });

    // 3. Parse and validate the flow
    let _dag = DagGraph::from_json(&flow_def)?;
    println!("✓ Flow definition parsed and validated\n");

    // 4. Create engine and execute
    let engine = FlowEngine::new(registry);

    // Set up initial variables (including conversation context)
    let mut variables = HashMap::new();
    variables.insert("conversation_id".to_string(), json!("conv-12345"));

    println!("▶ Starting flow execution...\n");

    // Convert DagGraph to JSON for engine.start()
    let flow_json = serde_json::to_value(&flow_def)?;
    let execution_id = engine.start(&flow_json, variables).await?;

    // 5. Poll for completion
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let state = engine.state(execution_id).await?;

        match state {
            a3s_flow::ExecutionState::Completed(result) => {
                println!("\n✓ Flow completed successfully");
                println!("  Execution ID: {}", execution_id);
                println!(
                    "  Final outputs: {}",
                    serde_json::to_string_pretty(&result.outputs)?
                );
                break;
            }
            a3s_flow::ExecutionState::Failed(msg) => {
                println!("\n✗ Flow failed: {}", msg);
                return Err(msg.into());
            }
            a3s_flow::ExecutionState::Terminated => {
                println!("\n⚠ Flow terminated");
                return Ok(());
            }
            _ => continue,
        }
    }
    // 6. Demonstrate context awareness
    println!("\n📝 Context Awareness Demo:");
    println!("  - The MCP node read 'conversation_id' from shared context");
    println!("  - The MCP node wrote 'last_mcp_call' to shared context");
    println!("  - Other nodes in the flow can access this shared state");

    Ok(())
}
