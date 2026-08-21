use crate::core::{ModelResponse, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    async fn generate(&self, prompt: &str) -> Result<ModelResponse>;
    async fn health_check(&self) -> Result<bool>;
}
