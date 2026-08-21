use serde::{Deserialize, Serialize};

use crate::types::message::{ContentPart, Message, MessageContent, Role};
use crate::types::tool::ToolCall;

/// Reason the model stopped generating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    ToolUse,
    Length,
    ContentFilter,
    Error,
}

/// Token usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_tokens: Option<u32>,
}

/// A unified chat completion response.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub content: Vec<ContentPart>,
    pub finish_reason: FinishReason,
    pub usage: Usage,
    pub thinking: Option<String>,
    /// The raw JSON response from the provider for debugging.
    pub raw: serde_json::Value,
}

impl ChatResponse {
    /// Get the text content of the response.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|p| match p {
                ContentPart::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Get all tool calls from the response.
    pub fn tool_calls(&self) -> Vec<ToolCall> {
        self.content
            .iter()
            .filter_map(|p| match p {
                ContentPart::ToolUse {
                    id,
                    name,
                    arguments,
                } => Some(ToolCall {
                    id: id.clone(),
                    name: name.clone(),
                    arguments: arguments.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    /// Whether the model wants to call tools.
    pub fn has_tool_calls(&self) -> bool {
        self.content
            .iter()
            .any(|p| matches!(p, ContentPart::ToolUse { .. }))
    }

    /// Convert this response into an assistant Message for multi-turn conversations.
    pub fn to_assistant_message(&self) -> Message {
        Message {
            role: Role::Assistant,
            content: MessageContent::Parts(self.content.clone()),
            name: None,
        }
    }
}

/// Events emitted during streaming.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Text content delta.
    Delta { content: String },
    /// Thinking/reasoning delta.
    ThinkingDelta { content: String },
    /// Incremental tool call data.
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: String,
    },
    /// A complete tool call has been assembled.
    ToolCallComplete(ToolCall),
    /// Token usage (usually arrives at stream end).
    Usage(Usage),
    /// Stream finished.
    Done { finish_reason: FinishReason },
}
