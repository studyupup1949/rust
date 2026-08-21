use advance_router::{ChatRequest, Gateway, Message, ProviderConfig, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Gateway::builder();

    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        builder = builder.with_provider(advance_router::OpenAIProvider::new(
            ProviderConfig::new(key),
        ));
    }
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        builder = builder.with_provider(advance_router::AnthropicProvider::new(
            ProviderConfig::new(key),
        ));
    }

    let gateway = builder.build();
    let model = std::env::var("MODEL").unwrap_or_else(|_| "gpt-4o".to_string());

    let request = ChatRequest::new(
        &model,
        vec![
            Message::system("You are a helpful assistant."),
            Message::user("Write a haiku about Rust programming."),
        ],
    )
    .with_max_tokens(200);

    println!("Streaming from model: {}", model);
    let mut stream = gateway.stream(request).await?;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::Delta { content } => {
                print!("{}", content);
            }
            StreamEvent::ThinkingDelta { content } => {
                print!("[thinking: {}]", content);
            }
            StreamEvent::Usage(usage) => {
                println!("\n\n--- Usage: {} total tokens ---", usage.total_tokens);
            }
            StreamEvent::Done { finish_reason } => {
                println!("--- Done: {:?} ---", finish_reason);
            }
            _ => {}
        }
    }

    Ok(())
}
