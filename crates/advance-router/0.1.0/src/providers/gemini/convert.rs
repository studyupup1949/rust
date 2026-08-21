use serde_json::{json, Value};

use crate::error::RouterError;
use crate::types::message::{ContentPart, Message, MessageContent, Role};
use crate::types::request::ChatRequest;
use crate::types::response::{ChatResponse, FinishReason, Usage};
use crate::types::tool::ToolChoice;

/// Convert a unified ChatRequest into Gemini generateContent format.
pub fn to_gemini_request(request: &ChatRequest) -> Value {
    // Gemini uses "contents" with parts, roles are "user" and "model"
    let mut contents: Vec<Value> = Vec::new();
    let mut system_instruction = None;

    for msg in &request.messages {
        match msg.role {
            Role::System => {
                let text = msg.content.text();
                system_instruction = Some(json!({
                    "parts": [{ "text": text }]
                }));
            }
            Role::User | Role::Tool => {
                contents.push(convert_message(msg, "user"));
            }
            Role::Assistant => {
                contents.push(convert_message(msg, "model"));
            }
        }
    }

    let mut body = json!({
        "contents": contents,
    });

    if let Some(si) = system_instruction {
        body["systemInstruction"] = si;
    }

    // Generation config
    let mut gen_config = json!({});
    if let Some(temp) = request.temperature {
        gen_config["temperature"] = json!(temp);
    }
    if let Some(max) = request.max_tokens {
        gen_config["maxOutputTokens"] = json!(max);
    }
    if let Some(top_p) = request.top_p {
        gen_config["topP"] = json!(top_p);
    }
    if let Some(stop) = &request.stop {
        gen_config["stopSequences"] = json!(stop);
    }
    if request.json_mode {
        gen_config["responseMimeType"] = json!("application/json");
        if let Some(schema) = &request.json_schema {
            gen_config["responseSchema"] = schema.clone();
        }
    }

    if gen_config.as_object().map_or(false, |o| !o.is_empty()) {
        body["generationConfig"] = gen_config;
    }

    // Tools (function calling)
    if !request.tools.is_empty() {
        let function_declarations: Vec<Value> = request
            .tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                })
            })
            .collect();

        body["tools"] = json!([{
            "functionDeclarations": function_declarations,
        }]);

        if let Some(choice) = &request.tool_choice {
            body["toolConfig"] = json!({
                "functionCallingConfig": {
                    "mode": match choice {
                        ToolChoice::Auto => "AUTO",
                        ToolChoice::Required => "ANY",
                        ToolChoice::None => "NONE",
                        ToolChoice::Named(_) => "AUTO",
                    }
                }
            });
        }
    }

    // Extra parameters
    for (key, value) in &request.extra {
        body[key] = value.clone();
    }

    body
}

fn convert_message(msg: &Message, role: &str) -> Value {
    let parts = match &msg.content {
        MessageContent::Text(text) => vec![json!({ "text": text })],
        MessageContent::Parts(parts) => parts
            .iter()
            .filter_map(|part| match part {
                ContentPart::Text { text } => Some(json!({ "text": text })),
                ContentPart::Image { url, data, media_type } => {
                    if let Some(data) = data {
                        Some(json!({
                            "inlineData": {
                                "mimeType": media_type.as_deref().unwrap_or("image/png"),
                                "data": data,
                            }
                        }))
                    } else if let Some(url) = url {
                        Some(json!({
                            "fileData": {
                                "fileUri": url,
                            }
                        }))
                    } else {
                        None
                    }
                }
                ContentPart::ToolUse {
                    name, arguments, ..
                } => Some(json!({
                    "functionCall": {
                        "name": name,
                        "args": arguments,
                    }
                })),
                ContentPart::ToolResult {
                    tool_use_id: _,
                    content,
                    ..
                } => {
                    // Gemini uses functionResponse
                    let response_value: Value =
                        serde_json::from_str(content).unwrap_or(json!({ "result": content }));
                    Some(json!({
                        "functionResponse": {
                            "name": "", // Gemini requires name but we may not have it here
                            "response": response_value,
                        }
                    }))
                }
                ContentPart::Thinking { .. } => None,
            })
            .collect(),
    };

    json!({
        "role": role,
        "parts": parts,
    })
}

/// Convert a Gemini generateContent response into a unified ChatResponse.
pub fn from_gemini_response(json: Value) -> Result<ChatResponse, RouterError> {
    let candidate = &json["candidates"][0];
    let parts = candidate["content"]["parts"].as_array();

    let mut content = Vec::new();

    if let Some(parts) = parts {
        for part in parts {
            if let Some(text) = part["text"].as_str() {
                content.push(ContentPart::Text {
                    text: text.to_string(),
                });
            }
            if part["functionCall"].is_object() {
                let fc = &part["functionCall"];
                content.push(ContentPart::ToolUse {
                    id: format!("call_{}", fc["name"].as_str().unwrap_or("")),
                    name: fc["name"].as_str().unwrap_or("").to_string(),
                    arguments: fc["args"].clone(),
                });
            }
        }
    }

    let finish_reason = match candidate["finishReason"].as_str() {
        Some("STOP") => FinishReason::Stop,
        Some("MAX_TOKENS") => FinishReason::Length,
        Some("SAFETY") => FinishReason::ContentFilter,
        Some("TOOL_CALLS") | Some("FUNCTION_CALL") => FinishReason::ToolUse,
        _ => FinishReason::Stop,
    };

    let usage = if json["usageMetadata"].is_object() {
        Usage {
            prompt_tokens: json["usageMetadata"]["promptTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
            completion_tokens: json["usageMetadata"]["candidatesTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
            total_tokens: json["usageMetadata"]["totalTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
            cache_read_tokens: json["usageMetadata"]["cachedContentTokenCount"]
                .as_u64()
                .map(|v| v as u32),
            cache_write_tokens: None,
            thinking_tokens: None,
        }
    } else {
        Usage::default()
    };

    Ok(ChatResponse {
        id: String::new(),
        model: json["modelVersion"].as_str().unwrap_or("").to_string(),
        content,
        finish_reason,
        usage,
        thinking: None,
        raw: json,
    })
}
