pub mod anthropic;
pub mod ollama;
pub mod openai;
pub mod traits;

pub use anthropic::AnthropicProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use traits::Provider;

use crate::core::{error::AdversariaError, Config, Result};
use std::sync::Arc;

pub fn create_provider(provider_name: &str, config: &Config) -> Result<Arc<dyn Provider>> {
    let provider_config = config.providers.get(provider_name).ok_or_else(|| {
        AdversariaError::Provider(format!("Provider '{}' not found in config", provider_name))
    })?;

    let provider: Arc<dyn Provider> = match provider_name {
        "openai" => Arc::new(OpenAIProvider::new(provider_config.clone())?),
        "anthropic" => Arc::new(AnthropicProvider::new(provider_config.clone())?),
        "ollama" => Arc::new(OllamaProvider::new(provider_config.clone())?),
        _ => {
            return Err(AdversariaError::Provider(format!(
                "Unknown provider: {}",
                provider_name
            )))
        }
    };

    Ok(provider)
}
