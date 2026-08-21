use serde_json::{json, Value};

use crate::error::RouterError;
use crate::types::message::{ContentPart, Message, MessageContent, Role};
use crate::types::request::ChatRequest;
use crate::types::response::{ChatResponse, FinishReason, Usage};
use crate::types::tool::ToolChoice;

use super::config::OpenAICompatConfig;

/// Convert a unified ChatRequest into OpenAI-format JSON body.
pub fn to_openai_request(request: &ChatRequest, config: &OpenAICompatConfig) -> Value {
    let messages: Vec<Value> = request
        .messages
        .iter()
        .map(|msg| convert_message(msg))
        .collect();

    let mut body = json!({
        "model": request.model,
        "messages": messages,
    });

    if let Some(temp) = request.temperature {
        body["temperature"] = json!(temp);
    }
    if let Some(max) = request.max_tokens {
        body["max_tokens"] = json!(max);
    }
    if let Some(top_p) = request.top_p {
        body["top_p"] = json!(top_p);
    }
    if let Some(stop) = &request.stop {
        body["stop"] = json!(stop);
    }

    // Tools
    if !request.tools.is_empty() && config.supports_tools {
        let tools: Vec<Value> = request
            .tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    }
                })
            })
            .collect();
        body["tools"] = json!(tools);

        if let Some(choice) = &request.tool_choice {
            body["tool_choice"] = match choice {
                ToolChoice::Auto => json!("auto"),
                ToolChoice::Required => json!("required"),
                ToolChoice::None => json!("none"),
                ToolChoice::Named(name) => json!({
                    "type": "function",
                    "function": { "name": name }
                }),
            };
        }
    }

    // JSON mode
    if request.json_mode && config.supports_json_mode {
        if let Some(schema) = &request.json_schema {
            body["response_format"] = json!({
                "type": "json_schema",
                "json_schema": schema,
            });
        } else {
            body["response_format"] = json!({ "type": "json_object" });
        }
    }

    // Stream
    if request.stream {
        body["stream"] = json!(true);
        body["stream_options"] = json!({ "include_usage": true });
    }

    // Extra provider-specific parameters
    for (key, value) in &request.extra {
        body[key] = value.clone();
    }

    body
}

fn convert_message(msg: &Message) -> Value {
    let role = match msg.role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    };

    match &msg.content {
        MessageContent::Text(text) => {
            let mut m = json!({ "role": role, "content": text });
            if let Some(name) = &msg.name {
                m["name"] = json!(name);
            }
            m
        }
        MessageContent::Parts(parts) => {
            // Check if it's a tool result message
            if msg.role == Role::Tool {
                if let Some(ContentPart::ToolResult {
                    tool_use_id,
                    content,
                    ..
                }) = parts.first()
                {
                    return json!({
                        "role": "tool",
                        "tool_call_id": tool_use_id,
                        "content": content,
                    });
                }
            }

            // Check if assistant message with tool calls
            if msg.role == Role::Assistant {
                let tool_uses: Vec<&ContentPart> = parts
                    .iter()
                    .filter(|p| matches!(p, ContentPart::ToolUse { .. }))
                    .collect();

                if !tool_uses.is_empty() {
                    let text_content: String = parts
                        .iter()
                        .filter_map(|p| match p {
                            ContentPart::Text { text } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect();

                    let tool_calls: Vec<Value> = tool_uses
                        .iter()
                        .map(|p| match p {
                            ContentPart::ToolUse {
                                id,
                                name,
                                arguments,
                            } => json!({
                                "id": id,
                                "type": "function",
                                "function": {
                                    "name": name,
                                    "arguments": arguments.to_string(),
                                }
                            }),
                            _ => unreachable!(),
                        })
                        .collect();

                    let mut m = json!({
                        "role": "assistant",
                        "tool_calls": tool_calls,
                    });
                    if !text_content.is_empty() {
                        m["content"] = json!(text_content);
                    }
                    return m;
                }
            }

            // General multimodal content
            let content: Vec<Value> = parts
                .iter()
                .filter_map(|part| match part {
                    ContentPart::Text { text } => Some(json!({
                        "type": "text",
                        "text": text,
                    })),
                    ContentPart::Image { url, data, media_type } => {
                        let image_url = if let Some(url) = url {
                            json!({ "url": url })
                        } else if let Some(data) = data {
                            let mt = media_type.as_deref().unwrap_or("image/png");
                            json!({ "url": format!("data:{};base64,{}", mt, data) })
                        } else {
                            return None;
                        };
                        Some(json!({
                            "type": "image_url",
                            "image_url": image_url,
                        }))
                    }
                    _ => None,
                })
                .collect();

            json!({ "role": role, "content": content })
        }
    }
}

/// Convert an OpenAI-format JSON response into a unified ChatResponse.
pub fn from_openai_response(
    json: Value,
    config: &OpenAICompatConfig,
) -> Result<ChatResponse, RouterError> {
    let id = json["id"].as_str().unwrap_or("").to_string();
    let model = json["model"].as_str().unwrap_or("").to_string();

    let choice = &json["choices"][0];

    let finish_reason = match choice["finish_reason"].as_str() {
        Some("stop") => FinishReason::Stop,
        Some("tool_calls") => FinishReason::ToolUse,
        Some("length") => FinishReason::Length,
        Some("content_filter") => FinishReason::ContentFilter,
        _ => FinishReason::Stop,
    };

    let message = &choice["message"];
    let mut content = Vec::new();

    // Text content
    if let Some(text) = message["content"].as_str() {
        if !text.is_empty() {
            content.push(ContentPart::Text {
                text: text.to_string(),
            });
        }
    }

    // Tool calls
    if let Some(tool_calls) = message["tool_calls"].as_array() {
        for tc in tool_calls {
            let id = tc["id"].as_str().unwrap_or("").to_string();
            let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
            let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
            let arguments: Value =
                serde_json::from_str(args_str).unwrap_or(Value::Object(Default::default()));

            content.push(ContentPart::ToolUse {
                id,
                name,
                arguments,
            });
        }
    }

    // Reasoning/thinking content (e.g. DeepSeek)
    let thinking = config
        .reasoning_field
        .as_ref()
        .and_then(|field| message[field].as_str())
        .map(|s| s.to_string());

    // Usage
    let usage = if json["usage"].is_object() {
        Usage {
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
        }
    } else {
        Usage::default()
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
