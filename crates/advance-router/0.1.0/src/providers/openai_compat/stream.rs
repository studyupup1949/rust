use serde_json::Value;

use crate::types::response::{FinishReason, StreamEvent, Usage};

use super::config::OpenAICompatConfig;

/// Parse an OpenAI streaming chunk JSON into a StreamEvent.
pub fn parse_stream_chunk(
    json: &Value,
    config: &OpenAICompatConfig,
) -> Vec<StreamEvent> {
    let mut events = Vec::new();

    // Usage chunk (at end of stream with stream_options.include_usage)
    if json["usage"].is_object() && !json["usage"].is_null() {
        events.push(StreamEvent::Usage(Usage {
            prompt_tokens: json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
            cache_read_tokens: json["usage"]["prompt_tokens_details"]["cached_tokens"]
                .as_u64()
                .map(|v| v as u32),
            cache_write_tokens: None,
            thinking_tokens: json["usage"]["completion_tokens_details"]["reasoning_tokens"]
                .as_u64()
                .map(|v| v as u32),
        }));
    }

    let choices = match json["choices"].as_array() {
        Some(c) => c,
        None => return events,
    };

    for choice in choices {
        let delta = &choice["delta"];

        // Text content delta
        if let Some(content) = delta["content"].as_str() {
            if !content.is_empty() {
                events.push(StreamEvent::Delta {
                    content: content.to_string(),
                });
            }
        }

        // Reasoning/thinking content delta (e.g. DeepSeek)
        if let Some(field) = &config.reasoning_field {
            if let Some(thinking) = delta[field].as_str() {
                if !thinking.is_empty() {
                    events.push(StreamEvent::ThinkingDelta {
                        content: thinking.to_string(),
                    });
                }
            }
        }

        // Tool call deltas
        if let Some(tool_calls) = delta["tool_calls"].as_array() {
            for tc in tool_calls {
                let index = tc["index"].as_u64().unwrap_or(0) as usize;
                let id = tc["id"].as_str().map(|s| s.to_string());
                let name = tc["function"]["name"].as_str().map(|s| s.to_string());
                let arguments_delta = tc["function"]["arguments"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                events.push(StreamEvent::ToolCallDelta {
                    index,
                    id,
                    name,
                    arguments_delta,
                });
            }
        }

        // Finish reason
        if let Some(reason) = choice["finish_reason"].as_str() {
            let finish_reason = match reason {
                "stop" => FinishReason::Stop,
                "tool_calls" => FinishReason::ToolUse,
                "length" => FinishReason::Length,
                "content_filter" => FinishReason::ContentFilter,
                _ => FinishReason::Stop,
            };
            events.push(StreamEvent::Done { finish_reason });
        }
    }

    events
}
