use serde::{Deserialize, Serialize};

/// A request to generate embeddings.
#[derive(Debug, Clone)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: Vec<String>,
    pub dimensions: Option<usize>,
}

/// A response containing embedding vectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub model: String,
    pub embeddings: Vec<Vec<f32>>,
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}
