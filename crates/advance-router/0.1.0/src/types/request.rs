use std::collections::HashMap;

use crate::types::message::Message;
use crate::types::tool::{ToolChoice, ToolDefinition};

/// Configuration for extended thinking / reasoning mode.
#[derive(Debug, Clone)]
pub struct ExtendedThinking {
    pub enabled: bool,
    pub budget_tokens: Option<u32>,
}

/// A unified chat completion request that works across all providers.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolDefinition>,
    pub tool_choice: Option<ToolChoice>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
    pub json_mode: bool,
    pub json_schema: Option<serde_json::Value>,
    pub extended_thinking: Option<ExtendedThinking>,
    /// Provider-specific passthrough parameters.
    pub extra: HashMap<String, serde_json::Value>,
}

impl ChatRequest {
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            tools: Vec::new(),
            tool_choice: None,
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            stream: false,
            json_mode: false,
            json_schema: None,
            extended_thinking: None,
            extra: HashMap::new(),
        }
    }

    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn with_json_mode(mut self, json_mode: bool) -> Self {
        self.json_mode = json_mode;
        self
    }

    pub fn with_json_schema(mut self, schema: serde_json::Value) -> Self {
        self.json_schema = Some(schema);
        self.json_mode = true;
        self
    }

    pub fn with_extended_thinking(mut self, budget_tokens: Option<u32>) -> Self {
        self.extended_thinking = Some(ExtendedThinking {
            enabled: true,
            budget_tokens,
        });
        self
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }
}
