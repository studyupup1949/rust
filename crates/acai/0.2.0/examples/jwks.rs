use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use acai::server::jwks::{JwksManager, KeyPairConfig};
use acai::server::make_typed_handler;
use acai::{
    AgentAuthentication, AgentCapabilities, AgentCard, AgentProvider, AgentSkill, Claims,
    JsonRpcError, MethodRouter, Server, ServerConfig, TokenData,
};
use serde::{Deserialize, Serialize};
use tokio::signal;
use tokio::time::sleep;
use uuid::Uuid;

/// Example message parameters
#[derive(Debug, Serialize, Deserialize)]
struct EchoParams {
    message: String,
}

/// Example message response
#[derive(Debug, Serialize, Deserialize)]
struct EchoResponse {
    message: String,
    authenticated: bool,
    user: Option<String>,
}

/// Authenticated message parameters
#[derive(Debug, Serialize, Deserialize)]
struct AuthenticatedParams {
    message: String,
    // Optional token - if not provided, will look for it in the request headers
    token: Option<String>,
}

/// Authenticated message response
#[derive(Debug, Serialize, Deserialize)]
struct AuthenticatedResponse {
    message: String,
    authenticated: bool,
    user: String,
    issued_at: u64,
    expires_at: u64,
}

/// Key info request parameters
#[derive(Debug, Serialize, Deserialize)]
struct KeyInfoParams;

/// Key info response
#[derive(Debug, Serialize, Deserialize)]
struct KeyInfoResponse {
    key_id: String,
    total_keys: usize,
}

/// New token request
#[derive(Debug, Serialize, Deserialize)]
struct NewTokenParams {
    subject: String,
    expiration_seconds: Option<u64>,
    custom_claims: Option<HashMap<String, serde_json::Value>>,
}

/// New token response
#[derive(Debug, Serialize, Deserialize)]
struct NewTokenResponse {
    token: String,
    expires_at: u64,
}

// The extract_token function has been removed as we now directly access params.token in the handler

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a JWKS manager for authentication
    let config = KeyPairConfig {
        name: "test".to_string(),
        private_key_path: "tests/test.key".to_string(),
    };
    let jwks_manager = Arc::new(JwksManager::new(vec![config])?);

    // Create a counter for demonstration purposes
    let request_counter = Arc::new(Mutex::new(0));

    // Schedule key information task (every 20 seconds for demo)
    let jwks_for_info = jwks_manager.clone();
    tokio::spawn(async move {
        loop {
            // Sleep for 20 seconds
            sleep(Duration::from_secs(20)).await;

            println!("Active key count: {}", jwks_for_info.key_count());
            println!("Active key IDs: {:?}", jwks_for_info.list_key_ids());
        }
    });

    // Create a simple echo handler (unauthenticated)
    async fn handle_echo(_: Arc<()>, params: EchoParams) -> Result<EchoResponse, JsonRpcError> {
        // Simply echo the message back
        let message = params.message.clone();
        Ok(EchoResponse {
            message,
            authenticated: false,
            user: None,
        })
    }

    let echo_handler = make_typed_handler(Arc::new(()), handle_echo);

    // Create an authenticated echo handler state
    struct AuthHandlerState {
        jwks: Arc<JwksManager>,
        counter: Arc<Mutex<i32>>,
    }

    let auth_state = AuthHandlerState {
        jwks: jwks_manager.clone(),
        counter: request_counter.clone(),
    };

    async fn handle_auth(
        state: Arc<AuthHandlerState>,
        params: AuthenticatedParams,
    ) -> Result<AuthenticatedResponse, JsonRpcError> {
        // Extract token from params
        let token = match params.token {
            Some(t) => t,
            None => {
                return Err(JsonRpcError::invalid_request(
                    "No authentication token provided",
                ));
            }
        };

        // Validate the token
        let token_data: TokenData<Claims> = match state.jwks.validate_token(&token) {
            Ok(data) => data,
            Err(e) => {
                return Err(JsonRpcError::internal_error(format!(
                    "Invalid token: {:?}",
                    e
                )));
            }
        };

        // Increment the request counter
        let count = {
            let mut counter = state.counter.lock().unwrap();
            *counter += 1;
            *counter
        };

        // Return authenticated response
        Ok(AuthenticatedResponse {
            message: format!("Echo: {} (request #{})", params.message, count),
            authenticated: true,
            user: token_data.claims.sub,
            issued_at: token_data.claims.iat,
            expires_at: token_data.claims.exp,
        })
    }

    let auth_handler = make_typed_handler(Arc::new(auth_state), handle_auth);

    // Handle key info requests
    async fn handle_key_info(
        jwks: Arc<JwksManager>,
        _params: KeyInfoParams,
    ) -> Result<KeyInfoResponse, JsonRpcError> {
        // Get key IDs
        let keys = jwks.list_key_ids();
        let key_id = keys
            .first()
            .ok_or_else(|| JsonRpcError::internal_error("No keys available"))?
            .clone();

        let total_keys = jwks.key_count();

        Ok(KeyInfoResponse { key_id, total_keys })
    }

    let key_info_handler = make_typed_handler(jwks_manager.clone(), handle_key_info);

    // Handler for generating new tokens
    async fn handle_token(
        jwks: Arc<JwksManager>,
        params: NewTokenParams,
    ) -> Result<NewTokenResponse, JsonRpcError> {
        // Get current time for token timestamps
        let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => time.as_secs(),
            Err(_) => return Err(JsonRpcError::internal_error("Failed to get current time")),
        };

        // Set token expiration (default 1 hour)
        let expires_in = params.expiration_seconds.unwrap_or(3600);
        let expiration = current_time + expires_in;

        // Create claim set
        let mut custom = HashMap::new();
        if let Some(claims) = params.custom_claims {
            custom.extend(claims);
        }

        let claims = Claims {
            sub: params.subject,
            iat: current_time,
            exp: expiration,
            jti: Uuid::new_v4().to_string(),
            custom,
        };

        // Generate the token
        let token = match jwks.generate_token(claims) {
            Ok(token) => token,
            Err(e) => {
                return Err(JsonRpcError::internal_error(format!(
                    "Failed to generate token: {:?}",
                    e
                )));
            }
        };

        Ok(NewTokenResponse {
            token,
            expires_at: expiration,
        })
    }

    let token_handler = make_typed_handler(jwks_manager.clone(), handle_token);

    // Create a router with our handlers
    let mut router = MethodRouter::new();
    router.register("echo", echo_handler);
    router.register("authenticated", auth_handler);
    router.register("key_info", key_info_handler);
    router.register("new_token", token_handler);

    // Create an agent card for discovery
    let agent_card = AgentCard {
        name: "JWKS Example Agent".to_string(),
        description: Some(
            "An example agent that demonstrates JWKS for JWT authentication".to_string(),
        ),
        url: "http://localhost:8080".to_string(),
        provider: Some(AgentProvider {
            organization: "Example Provider".to_string(),
            url: Some("https://example.com".to_string()),
        }),
        authentication: Some(AgentAuthentication {
            schemes: vec!["bearer".to_string()],
            credentials: None,
        }),
        version: "1.0.0".to_string(),
        documentation_url: Some("https://example.com/docs".to_string()),
        capabilities: AgentCapabilities {
            streaming: true,
            push_notifications: true,
            state_transition_history: false,
        },
        default_input_modes: vec!["text".to_string()],
        default_output_modes: vec!["text".to_string()],
        skills: vec![
            AgentSkill {
                id: "echo".to_string(),
                name: "Echo Service".to_string(),
                description: Some("Echoes a message back to the client (no auth)".to_string()),
                examples: Some(vec!["Echo hello".to_string()]),
                input_modes: Some(vec!["text".to_string()]),
                output_modes: Some(vec!["text".to_string()]),
                tags: None,
            },
            AgentSkill {
                id: "authenticated".to_string(),
                name: "Authenticated Echo".to_string(),
                description: Some("Echoes a message with authentication".to_string()),
                examples: Some(vec!["Authenticated hello".to_string()]),
                input_modes: Some(vec!["text".to_string()]),
                output_modes: Some(vec!["text".to_string()]),
                tags: None,
            },
            AgentSkill {
                id: "key_info".to_string(),
                name: "Key Information".to_string(),
                description: Some("Provides information about JWKS keys".to_string()),
                examples: None,
                input_modes: Some(vec!["text".to_string()]),
                output_modes: Some(vec!["text".to_string()]),
                tags: None,
            },
            AgentSkill {
                id: "new_token".to_string(),
                name: "Token Generator".to_string(),
                description: Some("Generates a new JWT token".to_string()),
                examples: None,
                input_modes: Some(vec!["text".to_string()]),
                output_modes: Some(vec!["text".to_string()]),
                tags: None,
            },
        ],
    };

    // Generate a test token
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        sub: "test-user".to_string(),
        iat: current_time,
        exp: current_time + 3600, // Valid for 1 hour
        jti: Uuid::new_v4().to_string(),
        custom: std::collections::HashMap::new(),
    };

    let token = jwks_manager.generate_token(claims)?;

    // Print test token and usage information
    println!("\n=== JWKS Demo API ===");
    println!("\nGenerated test token: {}", token);
    println!("\nExample curl commands:");
    println!("\n1. Public endpoint (no auth):");
    println!("curl -X POST -H \"Content-Type: application/json\" \\");
    println!(
        "  -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"echo\",\"params\":{{\"message\":\"Hello world\"}}}}' \\"
    );
    println!("  http://localhost:8080/jsonrpc");

    println!("\n2. Authenticated endpoint:");
    println!(
        "curl -X POST -H \"Content-Type: application/json\" -H \"Authorization: Bearer {}\" \\",
        token
    );
    println!(
        "  -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"authenticated\",\"params\":{{\"message\":\"Hello authenticated world\"}}}}' \\"
    );
    println!("  http://localhost:8080/jsonrpc");

    println!("\n3. Generate a new token:");
    println!("curl -X POST -H \"Content-Type: application/json\" \\");
    println!(
        "  -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"new_token\",\"params\":{{\"subject\":\"user123\",\"expiration_seconds\":7200}}}}' \\"
    );
    println!("  http://localhost:8080/jsonrpc");

    println!("\n4. Get key info:");
    println!("curl -X POST -H \"Content-Type: application/json\" \\");
    println!("  -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"key_info\",\"params\":null}}' \\");
    println!("  http://localhost:8080/jsonrpc");

    println!("\n5. View JWKS endpoint:");
    println!("curl http://localhost:8080/.well-known/jwks.json");

    println!("\nNote: Key information is logged every 20 seconds for demonstration purposes.");
    println!("===========================\n");

    // Create a server with JWKS support
    let server = Server::new(ServerConfig::default(), Arc::new(router))
        .with_agent_card(agent_card)
        .with_jwks_manager(jwks_manager);

    // Wait for Ctrl+C or server error
    let _server_task = tokio::spawn(async move {
        if let Err(e) = server.serve().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Wait for Ctrl+C
    match signal::ctrl_c().await {
        Ok(()) => {
            println!("Shutting down...");
            // server_task will be dropped when this function returns
        }
        Err(err) => {
            eprintln!("Error waiting for Ctrl+C: {}", err);
        }
    }

    Ok(())
}
