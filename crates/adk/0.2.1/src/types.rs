use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a generic context that can be used by agents and tools
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Additional data stored in the context
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn with_data(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(value) = serde_json::to_value(value) {
            self.data.insert(key.into(), value);
        }
        self
    }
}

/// Represents the context for a single run of an agent
#[derive(Debug, Clone)]
pub struct RunContext {
    /// The base context containing shared data
    pub context: Context,
    /// Messages exchanged during the run
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender (system, user, assistant, tool)
    pub role: String,
    /// The content of the message
    pub content: String,
    /// Optional name of the tool that generated this message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

impl RunContext {
    pub fn new(context: Context) -> Self {
        Self {
            context,
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, role: impl Into<String>, content: impl Into<String>) {
        self.messages.push(Message {
            role: role.into(),
            content: content.into(),
            tool_name: None,
        });
    }

    pub fn add_tool_message(&mut self, tool_name: impl Into<String>, content: impl Into<String>) {
        self.messages.push(Message {
            role: "tool".into(),
            content: content.into(),
            tool_name: Some(tool_name.into()),
        });
    }
}
