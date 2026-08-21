use serde_json::Value;

use crate::types::response::{FinishReason, StreamEvent, Usage};
use crate::types::tool::ToolCall;

/// Parse a Gemini streaming chunk into StreamEvents.
///
/// Gemini streaming returns complete candidate objects incrementally.
/// Each chunk contains the full current state of the response.
pub fn parse_stream_chunk(json: &Value) -> Vec<StreamEvent> {
    let mut events = Vec::new();

    let candidates = match json["candidates"].as_array() {
        Some(c) => c,
        None => return events,
    };

    for candidate in candidates {
        let parts = match candidate["content"]["parts"].as_array() {
            Some(p) => p,
            None => continue,
        };

        for part in parts {
            // Text content
            if let Some(text) = part["text"].as_str() {
                if !text.is_empty() {
                    events.push(StreamEvent::Delta {
                        content: text.to_string(),
                    });
                }
            }

            // Function call
            if part["functionCall"].is_object() {
                let fc = &part["functionCall"];
                let name = fc["name"].as_str().unwrap_or("").to_string();
                events.push(StreamEvent::ToolCallComplete(ToolCall {
                    id: format!("call_{}", name),
                    name,
                    arguments: fc["args"].clone(),
                }));
            }
        }

        // Finish reason
        if let Some(reason) = candidate["finishReason"].as_str() {
            let finish_reason = match reason {
                "STOP" => FinishReason::Stop,
                "MAX_TOKENS" => FinishReason::Length,
                "SAFETY" => FinishReason::ContentFilter,
                _ => FinishReason::Stop,
            };
            events.push(StreamEvent::Done { finish_reason });
        }
    }

    // Usage metadata
    if json["usageMetadata"].is_object() {
        events.push(StreamEvent::Usage(Usage {
            prompt_tokens: json["usageMetadata"]["promptTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
            completion_tokens: json["usageMetadata"]["candidatesTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
            total_tokens: json["usageMetadata"]["totalTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
            cache_read_tokens: None,
            cache_write_tokens: None,
            thinking_tokens: None,
        }));
    }

    events
}
