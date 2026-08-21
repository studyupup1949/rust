use std::sync::Arc;

use crate::error::{AgentError, AgentResult};
use crate::openai::Model;
use crate::tool::Tool;
use crate::types::{Context, RunContext};

/// An agent that can use tools and interact with a language model
pub struct Agent {
    /// The name of the agent
    name: String,
    /// The instructions for the agent (system prompt)
    instructions: Option<String>,
    /// The model to use for generating responses
    model: Arc<dyn Model>,
    /// The tools available to the agent
    tools: Vec<Arc<dyn Tool>>,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        name: impl Into<String>,
        instructions: Option<String>,
        model: Arc<dyn Model>,
        tools: Vec<Arc<dyn Tool>>,
    ) -> Self {
        Self {
            name: name.into(),
            instructions,
            model,
            tools,
        }
    }

    /// Run the agent with the given input
    pub async fn run(&self, input: impl Into<String>, context: Context) -> AgentResult<String> {
        let mut run_context = RunContext::new(context);

        // Add system instructions if provided
        if let Some(instructions) = &self.instructions {
            run_context.add_message("system", instructions);
        }

        // Add user input
        run_context.add_message("user", input);

        // Convert tools to slice of references
        let tools: Vec<&dyn Tool> = self.tools.iter().map(|tool| tool.as_ref()).collect();

        // Generate response
        self.model.generate_response(&mut run_context, &tools).await
    }

    /// Get the name of the agent
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the instructions for the agent
    pub fn instructions(&self) -> Option<&str> {
        self.instructions.as_deref()
    }

    /// Get the tools available to the agent
    pub fn tools(&self) -> &[Arc<dyn Tool>] {
        &self.tools
    }
}

/// Builder for creating agents
pub struct AgentBuilder {
    name: String,
    instructions: Option<String>,
    model: Option<Arc<dyn Model>>,
    tools: Vec<Arc<dyn Tool>>,
}

impl AgentBuilder {
    /// Create a new agent builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            instructions: None,
            model: None,
            tools: Vec::new(),
        }
    }

    /// Set the instructions for the agent
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Set the model for the agent
    pub fn model(mut self, model: Arc<dyn Model>) -> Self {
        self.model = Some(model);
        self
    }

    /// Add a tool to the agent
    pub fn add_tool(mut self, tool: Arc<dyn Tool>) -> Self {
        self.tools.push(tool);
        self
    }

    /// Build the agent
    pub fn build(self) -> AgentResult<Agent> {
        let model = self
            .model
            .ok_or_else(|| AgentError::ConfigurationError("Model not set".into()))?;
        Ok(Agent::new(self.name, self.instructions, model, self.tools))
    }
}
