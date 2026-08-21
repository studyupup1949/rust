use crate::llm::traits::{LLMProvider, LLMResponse};
use async_trait::async_trait;

#[derive(Debug)]
pub struct ClaudeCodeProvider {
    binary_path: String,
    working_dir: std::path::PathBuf,
}

impl ClaudeCodeProvider {
    pub fn new(working_dir: std::path::PathBuf) -> Self {
        ClaudeCodeProvider {
            binary_path: "claude".to_string(),
            working_dir,
        }
    }

    pub fn is_available() -> bool {
        std::process::Command::new("which")
            .arg("claude")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }
}

#[async_trait]
impl LLMProvider for ClaudeCodeProvider {
    async fn chat(
        &self,
        messages: &[crate::llm::traits::Message],
        _options: &crate::llm::traits::LLMOptions,
    ) -> Result<LLMResponse, String> {
        let prompt = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let output = tokio::process::Command::new(&self.binary_path)
            .args(&["--print", "--output-format", "json", "-p", prompt])
            .current_dir(&self.working_dir)
            .output()
            .await
            .map_err(|e| format!("Claude Code exec failed: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Claude Code failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("Claude Code parse failed: {}", e))?;

        Ok(LLMResponse {
            content: json["result"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            model: "claude-code".to_string(),
            usage: None,
        })
    }
}
