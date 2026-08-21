use serde_json::{json, Value};

use crate::error::RouterError;
use crate::types::message::{ContentPart, Message, MessageContent, Role};
use crate::types::request::ChatRequest;
use crate::types::response::{ChatResponse, FinishReason, Usage};
use crate::types::tool::ToolChoice;

/// Convert a unified ChatRequest into Anthropic Messages API format.
pub fn to_anthropic_request(request: &ChatRequest) -> Value {
    // Extract system message (Anthropic requires it as a top-level field)
    let system_content: Vec<Value> = request
        .messages
        .iter()
        .filter(|m| m.role == Role::System)
        .map(|m| match &m.content {
            MessageContent::Text(text) => json!({ "type": "text", "text": text }),
            MessageContent::Parts(parts) => {
                let texts: Vec<Value> = parts
                    .iter()
                    .filter_map(|p| match p {
                        ContentPart::Text { text } => {
                            Some(json!({ "type": "text", "text": text }))
                        }
                        _ => None,
                    })
                    .collect();
                json!(texts)
            }
        })
        .collect();

    // Non-system messages
    let messages: Vec<Value> = request
        .messages
        .iter()
        .filter(|m| m.role != Role::System)
        .map(convert_message)
        .collect();

    let mut body = json!({
        "model": request.model,
        "messages": messages,
    });

    if !system_content.is_empty() {
        if system_content.len() == 1 {
            body["system"] = system_content.into_iter().next().unwrap();
        } else {
            body["system"] = json!(system_content);
        }
    }

    // Max tokens (required by Anthropic)
    body["max_tokens"] = json!(request.max_tokens.unwrap_or(4096));

    if let Some(temp) = request.temperature {
        body["temperature"] = json!(temp);
    }
    if let Some(top_p) = request.top_p {
        body["top_p"] = json!(top_p);
    }
    if let Some(stop) = &request.stop {
        body["stop_sequences"] = json!(stop);
    }

    // Tools
    if !request.tools.is_empty() {
        let tools: Vec<Value> = request
            .tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.parameters,
                })
            })
            .collect();
        body["tools"] = json!(tools);

        if let Some(choice) = &request.tool_choice {
            body["tool_choice"] = match choice {
                ToolChoice::Auto => json!({ "type": "auto" }),
                ToolChoice::Required => json!({ "type": "any" }),
                ToolChoice::None => json!({ "type": "none" }),
                ToolChoice::Named(name) => json!({ "type": "tool", "name": name }),
            };
        }
    }

    // Extended thinking
    if let Some(thinking) = &request.extended_thinking {
        if thinking.enabled {
            body["thinking"] = json!({
                "type": "enabled",
                "budget_tokens": thinking.budget_tokens.unwrap_or(10000),
            });
        }
    }

    // Streaming
    if request.stream {
        body["stream"] = json!(true);
    }

    // Extra parameters
    for (key, value) in &request.extra {
        body[key] = value.clone();
    }

    body
}

fn convert_message(msg: &Message) -> Value {
    let role = match msg.role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "user", // Anthropic represents tool results as user messages
        Role::System => unreachable!("System messages handled separately"),
    };

    match &msg.content {
        MessageContent::Text(text) => {
            json!({ "role": role, "content": text })
        }
        MessageContent::Parts(parts) => {
            let content: Vec<Value> = parts
                .iter()
                .map(|part| match part {
                    ContentPart::Text { text } => {
                        json!({ "type": "text", "text": text })
                    }
                    ContentPart::Image {
                        url,
                        data,
                        media_type,
                    } => {
                        if let Some(data) = data {
                            json!({
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": media_type.as_deref().unwrap_or("image/png"),
                                    "data": data,
                                }
                            })
                        } else if let Some(url) = url {
                            json!({
                                "type": "image",
                                "source": {
                                    "type": "url",
                                    "url": url,
                                }
                            })
                        } else {
                            json!({ "type": "text", "text": "[missing image]" })
                        }
                    }
                    ContentPart::ToolUse {
                        id,
                        name,
                        arguments,
                    } => {
                        json!({
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": arguments,
                        })
                    }
                    ContentPart::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    } => {
                        json!({
                            "type": "tool_result",
                            "tool_use_id": tool_use_id,
                            "content": content,
                            "is_error": is_error,
                        })
                    }
                    ContentPart::Thinking { thinking } => {
                        json!({
                            "type": "thinking",
                            "thinking": thinking,
                        })
                    }
                })
                .collect();

            json!({ "role": role, "content": content })
        }
    }
}

/// Convert an Anthropic Messages API response into a unified ChatResponse.
pub fn from_anthropic_response(json: Value) -> Result<ChatResponse, RouterError> {
    let id = json["id"].as_str().unwrap_or("").to_string();
    let model = json["model"].as_str().unwrap_or("").to_string();

    let finish_reason = match json["stop_reason"].as_str() {
        Some("end_turn") => FinishReason::Stop,
        Some("tool_use") => FinishReason::ToolUse,
        Some("max_tokens") => FinishReason::Length,
        _ => FinishReason::Stop,
    };

    let mut content = Vec::new();
    let mut thinking = None;

    if let Some(blocks) = json["content"].as_array() {
        for block in blocks {
            match block["type"].as_str() {
                Some("text") => {
                    if let Some(text) = block["text"].as_str() {
                        content.push(ContentPart::Text {
                            text: text.to_string(),
                        });
                    }
                }
                Some("tool_use") => {
                    content.push(ContentPart::ToolUse {
                        id: block["id"].as_str().unwrap_or("").to_string(),
                        name: block["name"].as_str().unwrap_or("").to_string(),
                        arguments: block["input"].clone(),
                    });
                }
                Some("thinking") => {
                    if let Some(text) = block["thinking"].as_str() {
                        thinking = Some(text.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    let usage = Usage {
        prompt_tokens: json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
        completion_tokens: json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
        total_tokens: (json["usage"]["input_tokens"].as_u64().unwrap_or(0)
            + json["usage"]["output_tokens"].as_u64().unwrap_or(0)) as u32,
        cache_read_tokens: json["usage"]["cache_read_input_tokens"]
            .as_u64()
            .map(|v| v as u32),
        cache_write_tokens: json["usage"]["cache_creation_input_tokens"]
            .as_u64()
            .map(|v| v as u32),
        thinking_tokens: None,
    };

    Ok(ChatResponse {
        id,
        model,
        content,
        finish_reason,
        usage,
        thinking,
        raw: json,
    })
}
