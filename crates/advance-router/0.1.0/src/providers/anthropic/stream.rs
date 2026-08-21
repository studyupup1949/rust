use serde_json::Value;

use crate::types::response::{FinishReason, StreamEvent, Usage};
use crate::types::tool::ToolCall;

/// State machine for parsing Anthropic's streaming SSE events.
///
/// Anthropic uses a multi-event protocol:
/// - message_start: Contains message metadata and usage
/// - content_block_start: Announces a new content block (text, tool_use, thinking)
/// - content_block_delta: Incremental content for the current block
/// - content_block_stop: End of current block
/// - message_delta: Final message metadata (stop_reason, usage)
/// - message_stop: Stream complete
#[derive(Default)]
pub struct AnthropicStreamParser {
    /// Type of the current content block being streamed
    current_block_type: Option<String>,
    /// Tool use accumulator for current block
    current_tool_id: Option<String>,
    current_tool_name: Option<String>,
    current_tool_args: String,
    tool_index: usize,
}

impl AnthropicStreamParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a raw SSE event (the full JSON including "type" field).
    /// Returns zero or more StreamEvents.
    pub fn parse_event(&mut self, json: &Value) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        let event_type = match json["type"].as_str() {
            Some(t) => t,
            None => return events,
        };

        match event_type {
            "message_start" => {
                // Extract initial usage if present
                if let Some(usage) = json["message"]["usage"].as_object() {
                    let input = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    events.push(StreamEvent::Usage(Usage {
                        prompt_tokens: input as u32,
                        ..Default::default()
                    }));
                }
            }

            "content_block_start" => {
                let block = &json["content_block"];
                let block_type = block["type"].as_str().unwrap_or("text");
                self.current_block_type = Some(block_type.to_string());

                if block_type == "tool_use" {
                    self.current_tool_id = block["id"].as_str().map(|s| s.to_string());
                    self.current_tool_name = block["name"].as_str().map(|s| s.to_string());
                    self.current_tool_args.clear();

                    events.push(StreamEvent::ToolCallDelta {
                        index: self.tool_index,
                        id: self.current_tool_id.clone(),
                        name: self.current_tool_name.clone(),
                        arguments_delta: String::new(),
                    });
                }
            }

            "content_block_delta" => {
                let delta = &json["delta"];

                match delta["type"].as_str() {
                    Some("text_delta") => {
                        if let Some(text) = delta["text"].as_str() {
                            events.push(StreamEvent::Delta {
                                content: text.to_string(),
                            });
                        }
                    }
                    Some("thinking_delta") => {
                        if let Some(thinking) = delta["thinking"].as_str() {
                            events.push(StreamEvent::ThinkingDelta {
                                content: thinking.to_string(),
                            });
                        }
                    }
                    Some("input_json_delta") => {
                        if let Some(partial) = delta["partial_json"].as_str() {
                            self.current_tool_args.push_str(partial);
                            events.push(StreamEvent::ToolCallDelta {
                                index: self.tool_index,
                                id: None,
                                name: None,
                                arguments_delta: partial.to_string(),
                            });
                        }
                    }
                    _ => {}
                }
            }

            "content_block_stop" => {
                if self.current_block_type.as_deref() == Some("tool_use") {
                    let arguments = serde_json::from_str(&self.current_tool_args)
                        .unwrap_or(Value::Object(Default::default()));

                    events.push(StreamEvent::ToolCallComplete(ToolCall {
                        id: self.current_tool_id.take().unwrap_or_default(),
                        name: self.current_tool_name.take().unwrap_or_default(),
                        arguments,
                    }));

                    self.tool_index += 1;
                    self.current_tool_args.clear();
                }
                self.current_block_type = None;
            }

            "message_delta" => {
                let delta = &json["delta"];

                let finish_reason = match delta["stop_reason"].as_str() {
                    Some("end_turn") => FinishReason::Stop,
                    Some("tool_use") => FinishReason::ToolUse,
                    Some("max_tokens") => FinishReason::Length,
                    _ => FinishReason::Stop,
                };

                // Final usage
                if let Some(usage) = json["usage"].as_object() {
                    let output = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                    events.push(StreamEvent::Usage(Usage {
                        completion_tokens: output as u32,
                        ..Default::default()
                    }));
                }

                events.push(StreamEvent::Done { finish_reason });
            }

            "message_stop" => {
                // Stream is done. If we haven't emitted Done yet, emit it now.
            }

            "ping" | "error" => {
                // Ignore pings; errors are handled at HTTP level
            }

            _ => {}
        }

        events
    }
}
