use advance_router::{ChatRequest, Gateway, Message, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a gateway with multiple providers
    let mut builder = Gateway::builder();

    // Add OpenAI if API key is set
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        builder = builder.with_provider(advance_router::OpenAIProvider::new(
            ProviderConfig::new(key),
        ));
    }

    // Add Anthropic if API key is set
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        builder = builder.with_provider(advance_router::AnthropicProvider::new(
            ProviderConfig::new(key),
        ));
    }

    // Add Gemini if API key is set
    if let Ok(key) = std::env::var("GEMINI_API_KEY") {
        builder = builder.with_provider(advance_router::GeminiProvider::new(
            ProviderConfig::new(key),
        ));
    }

    let gateway = builder.build();

    // The gateway automatically routes based on model name prefix
    let model = std::env::var("MODEL").unwrap_or_else(|_| "gpt-4o".to_string());

    let request = ChatRequest::new(
        &model,
        vec![
            Message::system("You are a helpful assistant."),
            Message::user("What is the capital of France? Answer briefly."),
        ],
    )
    .with_max_tokens(100);

    println!("Sending request to model: {}", model);
    let response = gateway.generate(request).await?;

    println!("Response: {}", response.text());
    println!(
        "Usage: {} prompt + {} completion = {} total tokens",
        response.usage.prompt_tokens,
        response.usage.completion_tokens,
        response.usage.total_tokens,
    );

    Ok(())
}
