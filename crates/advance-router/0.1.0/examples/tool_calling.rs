use advance_router::{
    agent_loop, ChatRequest, Gateway, Message, ProviderConfig, ToolCall, ToolDefinition, ToolResult,
};
use serde_json::json;

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
    let model_name = std::env::var("MODEL").unwrap_or_else(|_| "gpt-4o".to_string());

    // Define a weather tool
    let weather_tool = ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get current weather for a city".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "City name"
                }
            },
            "required": ["city"]
        }),
    };

    let request = ChatRequest::new(
        &model_name,
        vec![Message::user(
            "What's the weather in Tokyo and Paris?",
        )],
    )
    .with_tools(vec![weather_tool])
    .with_max_tokens(1024);

    let model = gateway.model(&model_name)?;

    // Run agent loop with automatic tool execution
    let response = agent_loop(
        model.as_ref(),
        request,
        |tc: ToolCall| async move {
            println!("Tool called: {} with args: {}", tc.name, tc.arguments);

            // Simulate weather API response
            let city = tc.arguments["city"].as_str().unwrap_or("unknown");
            let weather = match city {
                "Tokyo" => "Sunny, 22°C",
                "Paris" => "Cloudy, 15°C",
                _ => "Unknown",
            };

            ToolResult {
                tool_call_id: tc.id,
                content: format!("Weather in {}: {}", city, weather),
                is_error: false,
            }
        },
        5, // max 5 rounds
    )
    .await?;

    println!("\nFinal response: {}", response.text());

    Ok(())
}
