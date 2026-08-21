//! Shared model factory for creating LLM models from configuration.
//!
//! Centralizes model creation logic for all supported providers,
//! eliminating duplication across agent modules.

use crate::models::ModelConfig;
use crate::{RalphError, Result};
use adk_rust::Llm;
use std::sync::Arc;

/// Create an LLM model from a [`ModelConfig`].
///
/// Supports all adk-rust 0.5.0 providers:
/// gemini, openai, anthropic, deepseek, groq, ollama,
/// fireworks, together, mistral, perplexity, cerebras, sambanova,
/// bedrock, azure-ai.
pub async fn create_model_from_config(config: &ModelConfig) -> Result<Arc<dyn Llm>> {
    use std::env;

    let model: Arc<dyn Llm> = match config.provider.to_lowercase().as_str() {
        "gemini" => {
            use adk_rust::model::GeminiModel;
            let api_key = env::var("GOOGLE_API_KEY")
                .or_else(|_| env::var("GEMINI_API_KEY"))
                .map_err(|_| {
                    RalphError::Configuration(
                        "GOOGLE_API_KEY or GEMINI_API_KEY environment variable not set".into(),
                    )
                })?;
            Arc::new(
                GeminiModel::new(api_key, &config.model_name)
                    .map_err(|e| RalphError::Model { provider: "gemini".into(), message: e.to_string() })?,
            )
        }
        "openai" => {
            use adk_rust::model::openai::{OpenAIClient, OpenAIConfig};
            let api_key = env_required("OPENAI_API_KEY")?;
            Arc::new(
                OpenAIClient::new(OpenAIConfig::new(api_key, &config.model_name))
                    .map_err(|e| RalphError::Model { provider: "openai".into(), message: e.to_string() })?,
            )
        }
        "anthropic" => {
            use adk_rust::model::anthropic::{AnthropicClient, AnthropicConfig};
            let api_key = env_required("ANTHROPIC_API_KEY")?;
            let mut cfg = AnthropicConfig::new(api_key, &config.model_name);
            if config.thinking_enabled {
                cfg = cfg.with_thinking(config.max_tokens as u32);
            }
            Arc::new(
                AnthropicClient::new(cfg)
                    .map_err(|e| RalphError::Model { provider: "anthropic".into(), message: e.to_string() })?,
            )
        }
        "deepseek" => {
            use adk_rust::model::deepseek::DeepSeekClient;
            let api_key = env_required("DEEPSEEK_API_KEY")?;
            if config.model_name.contains("reasoner") || config.model_name.contains("-r1") {
                Arc::new(DeepSeekClient::reasoner(api_key)
                    .map_err(|e| RalphError::Model { provider: "deepseek".into(), message: e.to_string() })?)
            } else {
                Arc::new(DeepSeekClient::chat(api_key)
                    .map_err(|e| RalphError::Model { provider: "deepseek".into(), message: e.to_string() })?)
            }
        }
        "groq" => {
            use adk_rust::model::groq::{GroqClient, GroqConfig};
            let api_key = env_required("GROQ_API_KEY")?;
            Arc::new(
                GroqClient::new(GroqConfig::new(api_key, &config.model_name))
                    .map_err(|e| RalphError::Model { provider: "groq".into(), message: e.to_string() })?,
            )
        }
        "ollama" => {
            use adk_rust::model::ollama::{OllamaModel, OllamaConfig};
            Arc::new(
                OllamaModel::new(OllamaConfig::new(&config.model_name))
                    .map_err(|e| RalphError::Model { provider: "ollama".into(), message: e.to_string() })?,
            )
        }
        // OpenAI-compatible providers via OpenAICompatible client
        "fireworks" => {
            use adk_rust::model::openai_compatible::{OpenAICompatible, OpenAICompatibleConfig};
            let api_key = env_required("FIREWORKS_API_KEY")?;
            Arc::new(
                OpenAICompatible::new(OpenAICompatibleConfig::fireworks(api_key, &config.model_name))
                    .map_err(|e| RalphError::Model { provider: "fireworks".into(), message: e.to_string() })?,
            )
        }
        "together" => {
            use adk_rust::model::openai_compatible::{OpenAICompatible, OpenAICompatibleConfig};
            let api_key = env_required("TOGETHER_API_KEY")?;
            Arc::new(
                OpenAICompatible::new(OpenAICompatibleConfig::together(api_key, &config.model_name))
                    .map_err(|e| RalphError::Model { provider: "together".into(), message: e.to_string() })?,
            )
        }
        "mistral" => {
            use adk_rust::model::openai_compatible::{OpenAICompatible, OpenAICompatibleConfig};
            let api_key = env_required("MISTRAL_API_KEY")?;
            Arc::new(
                OpenAICompatible::new(OpenAICompatibleConfig::mistral(api_key, &config.model_name))
                    .map_err(|e| RalphError::Model { provider: "mistral".into(), message: e.to_string() })?,
            )
        }
        "perplexity" => {
            use adk_rust::model::openai_compatible::{OpenAICompatible, OpenAICompatibleConfig};
            let api_key = env_required("PERPLEXITY_API_KEY")?;
            Arc::new(
                OpenAICompatible::new(OpenAICompatibleConfig::perplexity(api_key, &config.model_name))
                    .map_err(|e| RalphError::Model { provider: "perplexity".into(), message: e.to_string() })?,
            )
        }
        "cerebras" => {
            use adk_rust::model::openai_compatible::{OpenAICompatible, OpenAICompatibleConfig};
            let api_key = env_required("CEREBRAS_API_KEY")?;
            Arc::new(
                OpenAICompatible::new(OpenAICompatibleConfig::cerebras(api_key, &config.model_name))
                    .map_err(|e| RalphError::Model { provider: "cerebras".into(), message: e.to_string() })?,
            )
        }
        "sambanova" => {
            use adk_rust::model::openai_compatible::{OpenAICompatible, OpenAICompatibleConfig};
            let api_key = env_required("SAMBANOVA_API_KEY")?;
            Arc::new(
                OpenAICompatible::new(OpenAICompatibleConfig::sambanova(api_key, &config.model_name))
                    .map_err(|e| RalphError::Model { provider: "sambanova".into(), message: e.to_string() })?,
            )
        }
        "bedrock" => {
            use adk_rust::model::bedrock::{BedrockClient, BedrockConfig};
            let region = env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
            Arc::new(
                BedrockClient::new(BedrockConfig::new(&config.model_name, &region))
                    .await
                    .map_err(|e| RalphError::Model { provider: "bedrock".into(), message: e.to_string() })?,
            )
        }
        "azure-ai" => {
            use adk_rust::model::azure_ai::{AzureAIClient, AzureAIConfig};
            let api_key = env_required("AZURE_AI_API_KEY")?;
            let endpoint = env_required("AZURE_AI_ENDPOINT")?;
            Arc::new(
                AzureAIClient::new(AzureAIConfig::new(api_key, &endpoint, &config.model_name))
                    .map_err(|e| RalphError::Model { provider: "azure-ai".into(), message: e.to_string() })?,
            )
        }
        provider => {
            return Err(RalphError::Configuration(format!(
                "Unsupported model provider: '{}'. Supported: {}",
                provider,
                crate::models::SUPPORTED_PROVIDERS.join(", ")
            )));
        }
    };

    Ok(model)
}

/// Helper to read a required environment variable.
fn env_required(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| {
        RalphError::Configuration(format!("{} environment variable not set", name))
    })
}
