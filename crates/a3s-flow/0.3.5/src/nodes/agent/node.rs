//! `"agent"` node implementation.

use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};
use crate::nodes::llm::{build_jinja_context, render, ChatMessage, CompletionResult};

const DEFAULT_API_BASE: &str = "https://api.openai.com/v1";
const DEFAULT_TEMPERATURE: f64 = 0.7;
const DEFAULT_MAX_TURNS: usize = 10;
const CONVERSATION_KEY: &str = "conversation";

/// Agent node — multi-turn LLM with function calling.
pub struct AgentNode;

#[async_trait]
impl Node for AgentNode {
    fn node_type(&self) -> &str {
        "agent"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let config = AgentConfig::from_data(&ctx.data)?;
        let jinja_ctx = build_jinja_context(&ctx);

        // 1. Load existing conversation from context
        let conversation = load_conversation(&ctx)?;
        let mut messages: Vec<ChatMessage> = conversation;

        // 2. Inject system prompt if not already in conversation
        if let Some(ref sys) = config.system_prompt {
            if messages.iter().all(|m| m.role != "system") {
                let sys_content = render(sys, &jinja_ctx)?;
                messages.insert(
                    0,
                    ChatMessage {
                        role: "system".into(),
                        content: sys_content,
                    },
                );
            }
        }

        // 3. Render and append user message
        let user_content = render(&config.user_message_template, &jinja_ctx)?;
        messages.push(ChatMessage {
            role: "user".into(),
            content: user_content,
        });

        // 3. Main agent loop
        let mut tool_call_log: Vec<ToolCallEntry> = Vec::new();
        let mut final_text = String::new();
        let mut turns = 0;

        loop {
            turns += 1;
            if turns > config.max_turns {
                final_text = format!(
                    "[Agent reached max_turns ({}) without completing]",
                    config.max_turns
                );
                break;
            }

            // Call LLM (with tools if configured)
            let result = do_agent_completion(
                &config.api_base,
                &config.api_key,
                &config.model,
                messages.clone(),
                Some(config.temperature),
                config.max_tokens,
                &config.tools,
            )
            .await?;

            // Check finish reason
            if result.finish_reason == "tool_calls" {
                // Parse tool calls from response
                let tool_calls = parse_tool_calls(&result.text)?;
                for tc in tool_calls {
                    let tool_result = execute_tool(&tc.name, &tc.arguments).await;
                    let result_content = match &tool_result {
                        Ok(output) => output.clone(),
                        Err(e) => format!("error: {}", e),
                    };
                    tool_call_log.push(ToolCallEntry {
                        tool: tc.name.clone(),
                        args: tc.arguments,
                        result: result_content.clone(),
                    });
                    // Append tool result as assistant message with tool call id
                    messages.push(ChatMessage {
                        role: "assistant".into(),
                        content: result.text.clone(),
                    });
                    messages.push(ChatMessage {
                        role: "tool".into(),
                        content: result_content,
                    });
                }
                // Continue loop with updated messages
                continue;
            } else {
                // Stop — final response
                final_text = result.text;
                messages.push(ChatMessage {
                    role: "assistant".into(),
                    content: final_text.clone(),
                });
                break;
            }
        }

        // 4. Persist conversation back to context
        save_conversation(&ctx, &messages)?;

        Ok(json!({
            "text": final_text,
            "conversation": messages,
            "tool_calls": tool_call_log,
            "turns": turns,
        }))
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
struct AgentConfig {
    model: String,
    system_prompt: Option<String>,
    user_message_template: String,
    api_base: String,
    api_key: String,
    temperature: f64,
    max_tokens: Option<u64>,
    tools: Vec<Value>,
    max_turns: usize,
}

impl AgentConfig {
    fn from_data(data: &Value) -> Result<Self> {
        let model = data["model"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidDefinition("agent: missing data.model".into()))?
            .to_string();

        let user_message_template = data["user_message_template"]
            .as_str()
            .ok_or_else(|| {
                FlowError::InvalidDefinition("agent: missing data.user_message_template".into())
            })?
            .to_string();

        let system_prompt = data["system_prompt"].as_str().map(str::to_string);
        let api_base = data["api_base"]
            .as_str()
            .unwrap_or(DEFAULT_API_BASE)
            .trim_end_matches('/')
            .to_string();
        let api_key = data["api_key"].as_str().unwrap_or("").to_string();
        let temperature = data["temperature"].as_f64().unwrap_or(DEFAULT_TEMPERATURE);
        let max_tokens = data["max_tokens"].as_u64();
        let max_turns = data["max_turns"]
            .as_u64()
            .unwrap_or(DEFAULT_MAX_TURNS as u64) as usize;
        let tools = data["tools"].as_array().cloned().unwrap_or_default();

        Ok(Self {
            model,
            system_prompt,
            user_message_template,
            api_base,
            api_key,
            temperature,
            max_tokens,
            tools,
            max_turns,
        })
    }
}

// ── Conversation persistence ──────────────────────────────────────────────────

fn load_conversation(ctx: &ExecContext) -> Result<Vec<ChatMessage>> {
    let guard = ctx
        .context
        .read()
        .map_err(|_| FlowError::Internal("agent: failed to lock context".into()))?;
    if let Some(conv) = guard.get(CONVERSATION_KEY) {
        serde_json::from_value(conv.clone())
            .map_err(|e| FlowError::Internal(format!("agent: invalid conversation format: {}", e)))
    } else {
        Ok(Vec::new())
    }
}

fn save_conversation(ctx: &ExecContext, messages: &[ChatMessage]) -> Result<()> {
    let conv_value = serde_json::to_value(messages).map_err(|e| {
        FlowError::Internal(format!("agent: failed to serialize conversation: {}", e))
    })?;
    let mut guard = ctx
        .context
        .write()
        .map_err(|_| FlowError::Internal("agent: failed to lock context for write".into()))?;
    guard.insert(CONVERSATION_KEY.into(), conv_value);
    Ok(())
}

// ── LLM call with tools ─────────────────────────────────────────────────────

async fn do_agent_completion(
    api_base: &str,
    api_key: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    tools: &[Value],
) -> Result<CompletionResult> {
    // Inject system prompt as first message if present
    // (already handled by caller, but we handle it here for re-entrant calls)

    let mut body = json!({
        "model": model,
        "messages": messages,
        "temperature": temperature.unwrap_or(DEFAULT_TEMPERATURE),
    });
    if let Some(max_tok) = max_tokens {
        body["max_tokens"] = json!(max_tok);
    }
    if !tools.is_empty() {
        body["tools"] = json!(tools);
    }

    let url = format!("{}/chat/completions", api_base);
    let client = Client::new();
    let mut req = client.post(&url).json(&body);
    if !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }

    let response = req
        .send()
        .await
        .map_err(|e| FlowError::Internal(format!("agent: HTTP request failed: {}", e)))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| FlowError::Internal(format!("agent: failed to read response body: {}", e)))?;

    if !status.is_success() {
        return Err(FlowError::Internal(format!(
            "agent: API returned {}: {}",
            status, text
        )));
    }

    let resp: Value = serde_json::from_str(&text)
        .map_err(|e| FlowError::Internal(format!("agent: failed to parse response JSON: {}", e)))?;

    parse_agent_response(&resp)
}

fn parse_agent_response(resp: &Value) -> Result<CompletionResult> {
    // Try tool_calls format first
    if let Some(tools) = resp
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
    {
        if let Some(finish_reason) = tools.get("finish_reason").and_then(|f| f.as_str()) {
            if finish_reason == "tool_calls" {
                // Return the full assistant message for tool call parsing
                let message = tools.get("message").and_then(|m| m.as_str()).unwrap_or("");
                return Ok(CompletionResult {
                    text: message.to_string(),
                    model: resp["model"].as_str().unwrap_or("unknown").to_string(),
                    finish_reason: "tool_calls".to_string(),
                    prompt_tokens: resp
                        .pointer("/usage/prompt_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    completion_tokens: resp
                        .pointer("/usage/completion_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    total_tokens: resp
                        .pointer("/usage/total_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    reasoning: None,
                });
            }
        }
    }

    // Standard completion parse
    let text = resp
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            FlowError::Internal(
                "agent: unexpected response shape (missing choices[0].message.content)".into(),
            )
        })?
        .to_string();

    let finish_reason = resp
        .pointer("/choices/0/finish_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("stop")
        .to_string();

    Ok(CompletionResult {
        text,
        model: resp["model"].as_str().unwrap_or("unknown").to_string(),
        finish_reason,
        prompt_tokens: resp
            .pointer("/usage/prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        completion_tokens: resp
            .pointer("/usage/completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        total_tokens: resp
            .pointer("/usage/total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        reasoning: None,
    })
}

// ── Tool call parsing ────────────────────────────────────────────────────────

#[derive(Debug)]
struct ToolCall {
    name: String,
    arguments: Value,
}

fn parse_tool_calls(text: &str) -> Result<Vec<ToolCall>> {
    let data: Value = serde_json::from_str(text).map_err(|e| {
        FlowError::Internal(format!("agent: failed to parse tool call message: {}", e))
    })?;

    let calls = data
        .get("tool_calls")
        .and_then(|tc| tc.as_array())
        .ok_or_else(|| FlowError::Internal("agent: no tool_calls in message".into()))?
        .clone();

    let mut result = Vec::new();
    for call in calls {
        let name = call
            .get("function")
            .and_then(|f| f.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| FlowError::Internal("agent: malformed tool_call name".into()))?
            .to_string();
        let raw_args = call
            .get("function")
            .and_then(|f| f.get("arguments"));
        let arguments = match raw_args {
            Some(Value::String(s)) => {
                // Arguments is a JSON string — parse it
                serde_json::from_str(s).unwrap_or(json!({}))
            }
            Some(v) => v.clone(),
            None => json!({}),
        };
        result.push(ToolCall { name, arguments });
    }
    Ok(result)
}

// ── Tool execution ───────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
struct ToolCallEntry {
    tool: String,
    args: Value,
    result: String,
}

async fn execute_tool(tool_name: &str, arguments: &Value) -> Result<String> {
    match tool_name {
        "http_fetch" => execute_http_fetch(arguments).await,
        _ => Err(FlowError::InvalidDefinition(format!(
            "agent: unknown tool '{}'",
            tool_name
        ))),
    }
}

async fn execute_http_fetch(args: &Value) -> Result<String> {
    let url = args["url"]
        .as_str()
        .ok_or_else(|| FlowError::InvalidDefinition("http_fetch: missing url".into()))?;
    let method = args["method"].as_str().unwrap_or("GET");
    let headers = args["headers"].as_object().cloned().unwrap_or_default();
    let body = args.get("body").cloned();

    let client = Client::new();
    let mut req = match method.to_ascii_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        other => {
            return Err(FlowError::InvalidDefinition(format!(
                "http_fetch: unsupported method '{}'",
                other
            )));
        }
    };

    for (name, val) in headers {
        if let Some(v) = val.as_str() {
            req = req.header(name.as_str(), v);
        }
    }

    if let Some(b) = body {
        if !b.is_null() {
            req = req.json(&b);
        }
    }

    let response = req
        .send()
        .await
        .map_err(|e| FlowError::Internal(format!("http_fetch: request failed: {}", e)))?;

    let status = response.status().as_u16();
    let body_text = response
        .text()
        .await
        .map_err(|e| FlowError::Internal(format!("http_fetch: failed to read response: {}", e)))?;

    let body_json: Value = serde_json::from_str(&body_text).unwrap_or(Value::String(body_text));

    serde_json::to_string(&json!({
        "status": status,
        "body": body_json,
    }))
    .map_err(|e| FlowError::Internal(format!("http_fetch: failed to serialize result: {}", e)))
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tool_calls_single() {
        let msg = json!({
            "tool_calls": [
                {
                    "id": "call_123",
                    "function": {
                        "name": "http_fetch",
                        "arguments": "{\"url\": \"https://example.com\"}"
                    }
                }
            ]
        });
        let text = serde_json::to_string(&msg).unwrap();
        let calls = parse_tool_calls(&text).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "http_fetch");
        assert_eq!(calls[0].arguments["url"], "https://example.com");
    }

    #[test]
    fn agent_config_defaults() {
        let data = json!({
            "model": "gpt-4o-mini",
            "user_message_template": "Hello"
        });
        let cfg = AgentConfig::from_data(&data).unwrap();
        assert_eq!(cfg.max_turns, 10);
        assert!(cfg.tools.is_empty());
        assert_eq!(cfg.api_base, DEFAULT_API_BASE);
    }
}
