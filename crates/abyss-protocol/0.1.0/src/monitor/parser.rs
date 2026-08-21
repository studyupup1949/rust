use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Token 消耗事件，从 JSONL 解析得到
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEvent {
    pub timestamp: DateTime<Utc>,
    pub message_id: String,
    pub request_id: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
}

impl TokenEvent {
    /// 本次事件的总 token 数（input + output，不含 cache）
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    /// 去重用的唯一键
    pub fn dedup_key(&self) -> (String, String) {
        (self.message_id.clone(), self.request_id.clone())
    }
}

/// 从一行 JSONL 中解析 TokenEvent。
/// 跳过无 usage 数据的行、summary 类型、格式错误的行，返回 None。
pub fn parse_jsonl_line(line: &str) -> Option<TokenEvent> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let v: Value = serde_json::from_str(line).ok()?;

    // 跳过 summary 类型
    if v.get("type").and_then(|t| t.as_str()) == Some("summary") {
        return None;
    }

    // 提取 message.usage，无 usage 则跳过
    let message = v.get("message")?;
    let usage = message.get("usage")?;

    let input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64())?;
    let output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64())?;

    // cache tokens 可选，默认 0
    let cache_creation_tokens = usage
        .get("cache_creation_input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cache_read_tokens = usage
        .get("cache_read_input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // 提取 timestamp
    let timestamp_str = v.get("timestamp").and_then(|t| t.as_str())?;
    let timestamp = timestamp_str.parse::<DateTime<Utc>>().ok()?;

    // 提取 message.id
    let message_id = message
        .get("id")
        .and_then(|id| id.as_str())
        .unwrap_or("")
        .to_string();

    // 提取 requestId（顶层字段）
    let request_id = v
        .get("requestId")
        .and_then(|id| id.as_str())
        .unwrap_or("")
        .to_string();

    // 提取 message.model
    let model = message
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown")
        .to_string();

    Some(TokenEvent {
        timestamp,
        message_id,
        request_id,
        model,
        input_tokens,
        output_tokens,
        cache_creation_tokens,
        cache_read_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let line = r#"{"timestamp":"2024-01-15T10:30:00Z","type":"assistant","message":{"id":"msg_123","model":"claude-3","usage":{"input_tokens":500,"output_tokens":200,"cache_creation_input_tokens":100,"cache_read_input_tokens":50}},"requestId":"req_456"}"#;
        let event = parse_jsonl_line(line).unwrap();
        assert_eq!(event.message_id, "msg_123");
        assert_eq!(event.request_id, "req_456");
        assert_eq!(event.model, "claude-3");
        assert_eq!(event.input_tokens, 500);
        assert_eq!(event.output_tokens, 200);
        assert_eq!(event.cache_creation_tokens, 100);
        assert_eq!(event.cache_read_tokens, 50);
        assert_eq!(event.total_tokens(), 700);
    }

    #[test]
    fn test_skip_summary_type() {
        let line = r#"{"timestamp":"2024-01-15T10:30:00Z","type":"summary","message":{"id":"msg_1","model":"claude-3","usage":{"input_tokens":100,"output_tokens":50}},"requestId":"req_1"}"#;
        assert!(parse_jsonl_line(line).is_none());
    }

    #[test]
    fn test_skip_no_usage() {
        let line = r#"{"timestamp":"2024-01-15T10:30:00Z","type":"human","message":{"id":"msg_1","role":"user","content":"hello"},"requestId":"req_1"}"#;
        assert!(parse_jsonl_line(line).is_none());
    }

    #[test]
    fn test_skip_empty_line() {
        assert!(parse_jsonl_line("").is_none());
        assert!(parse_jsonl_line("  ").is_none());
    }

    #[test]
    fn test_skip_invalid_json() {
        assert!(parse_jsonl_line("not json at all").is_none());
    }

    #[test]
    fn test_cache_tokens_optional() {
        let line = r#"{"timestamp":"2024-01-15T10:30:00Z","type":"assistant","message":{"id":"msg_1","model":"claude-3","usage":{"input_tokens":100,"output_tokens":50}},"requestId":"req_1"}"#;
        let event = parse_jsonl_line(line).unwrap();
        assert_eq!(event.cache_creation_tokens, 0);
        assert_eq!(event.cache_read_tokens, 0);
    }

    #[test]
    fn test_dedup_key() {
        let line = r#"{"timestamp":"2024-01-15T10:30:00Z","message":{"id":"msg_A","model":"claude-3","usage":{"input_tokens":1,"output_tokens":1}},"requestId":"req_B"}"#;
        let event = parse_jsonl_line(line).unwrap();
        assert_eq!(event.dedup_key(), ("msg_A".to_string(), "req_B".to_string()));
    }
}
