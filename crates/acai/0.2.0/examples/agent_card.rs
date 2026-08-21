use std::sync::Arc;

use acai::{
    AgentCapabilities, AgentCard, AgentProvider, AgentSkill, JsonRpcError,
    client::{Client, ClientConfig},
    server::{MethodRouter, Server, ServerConfig, make_typed_handler},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Echo handler - just returns the input message
async fn echo_handler(
    _state: Arc<()>,
    message: String,
) -> std::result::Result<String, JsonRpcError> {
    Ok(message)
}

// Start server function
async fn start_server() -> Result<()> {
    // Create a router and register our echo method
    let mut router = MethodRouter::new();

    // Create a state for the handler
    let state = Arc::new(());

    // Register the echo handler using make_typed_handler
    router.register("echo", make_typed_handler(state, echo_handler));

    // Create server configuration
    let config = ServerConfig::new("127.0.0.1:3001")?;

    // Create an agent card
    let agent_card = AgentCard {
        name: "Echo Agent".to_string(),
        description: Some("A simple echo agent that responds with the same message".to_string()),
        url: "http://127.0.0.1:3001".to_string(),
        provider: Some(AgentProvider {
            organization: "Acai Project".to_string(),
            url: Some("https://github.com/rescrv/acai".to_string()),
        }),
        version: "0.1.0".to_string(),
        documentation_url: Some("https://github.com/rescrv/acai".to_string()),
        capabilities: AgentCapabilities {
            streaming: false,
            push_notifications: false,
            state_transition_history: false,
        },
        authentication: None,
        default_input_modes: vec!["text".to_string()],
        default_output_modes: vec!["text".to_string()],
        skills: vec![AgentSkill {
            id: "echo".to_string(),
            name: "Echo".to_string(),
            description: Some("Echoes back the input message".to_string()),
            tags: Some(vec!["echo".to_string(), "mirror".to_string()]),
            examples: Some(vec!["Echo this message".to_string()]),
            input_modes: Some(vec!["text".to_string()]),
            output_modes: Some(vec!["text".to_string()]),
        }],
    };

    // Create the server and add the agent card
    let server = Server::new(config, Arc::new(router)).with_agent_card(agent_card);

    println!("A2A server with agent card listening on http://127.0.0.1:3001");
    println!("Agent card available at http://127.0.0.1:3001/.well-known/agent.json");

    server.serve().await.map_err(|e| e.into())
}

async fn fetch_agent_card() -> Result<()> {
    // Connect to the server
    let config = ClientConfig::new("http://127.0.0.1:3001");
    let client = Client::new(config)?;

    // Fetch the agent card
    println!("Fetching agent card from http://127.0.0.1:3001/.well-known/agent.json");
    match client.fetch_agent_card().await {
        Ok(card) => {
            println!("\nAgent Card Retrieved:");
            println!("  Name: {}", card.name);
            println!("  Description: {}", card.description.unwrap_or_default());
            println!("  URL: {}", card.url);
            println!("  Version: {}", card.version);
            println!(
                "  Provider: {}",
                card.provider.map_or("None".to_string(), |p| p.organization)
            );

            println!("\n  Capabilities:");
            println!("    Streaming: {}", card.capabilities.streaming);
            println!(
                "    Push Notifications: {}",
                card.capabilities.push_notifications
            );
            println!(
                "    State Transition History: {}",
                card.capabilities.state_transition_history
            );

            println!("\n  Skills:");
            for skill in card.skills {
                println!("    - {} ({})", skill.name, skill.id);
                println!(
                    "      Description: {}",
                    skill.description.unwrap_or_default()
                );
                println!("      Tags: {:?}", skill.tags.unwrap_or_default());
                println!("      Examples: {:?}", skill.examples.unwrap_or_default());
            }

            println!("\nAgent card retrieved successfully!");
        }
        Err(err) => {
            println!("Failed to retrieve agent card: {}", err);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check for client or server mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "client" {
        fetch_agent_card().await
    } else {
        start_server().await
    }
}
