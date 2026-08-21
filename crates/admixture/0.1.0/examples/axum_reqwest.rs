//! Example: HTTP API Testing with Axum Server and Reqwest Client
//!
//! This example demonstrates how to use admixture for integration testing a real HTTP API.
//! It shows:
//! - Creating a service for an Axum HTTP server
//! - Creating a service for a reqwest HTTP client
//! - Starting the server on a random port
//! - Configuring the client to connect to the server
//! - Running tests against the API
//! - Clean shutdown of both services
//!
//! Run with: cargo run --example axum_reqwest

use admixture::service::{ServiceRunning, ServiceSetup};
use axum::{extract::Path, routing::get, Json, Router};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::TcpListener;

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: u64,
    pub name: String,
    pub email: String,
}

// ============================================================================
// Axum Server Service
// ============================================================================

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Failed to bind to address: {0}")]
    BindFailed(#[source] std::io::Error),
    
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
}

admixture::service! {
    AxumServer {
        error: ServerError,
        client: String,

        setup {
            port: u16,
        }

        running {
            base_url: String,
            shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
        }

        async fn start(self) -> Result<AxumServerRunning, ServerError> {
            let app = Router::new()
                .route("/health", get(health_handler))
                .route("/users/:id", get(get_user_handler));

            let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
            let listener = TcpListener::bind(addr).await.map_err(ServerError::BindFailed)?;
            let bound_addr = listener.local_addr().map_err(ServerError::BindFailed)?;
            let base_url = format!("http://{}", bound_addr);

            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

            tokio::spawn(async move {
                axum::serve(listener, app)
                    .with_graceful_shutdown(async { shutdown_rx.await.ok(); })
                    .await
                    .expect("server failed");
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            Ok(AxumServerRunning {
                base_url,
                shutdown_tx: Some(shutdown_tx),
            })
        }

        async fn client(&self) -> Result<String, ServerError> {
            Ok(self.base_url.clone())
        }

        async fn healthy(&self) -> Result<(), ServerError> {
            let client = reqwest::Client::new();
            let response = client.get(format!("{}/health", self.base_url)).send().await?;
            if response.status().is_success() {
                Ok(())
            } else {
                Err(ServerError::RequestFailed(response.error_for_status().unwrap_err()))
            }
        }

        async fn stop(&mut self) -> Result<(), ServerError> {
            if let Some(tx) = self.shutdown_tx.take() {
                let _ = tx.send(());
            }
            Ok(())
        }
    }
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
    })
}

async fn get_user_handler(Path(id): Path<u64>) -> Json<UserResponse> {
    Json(UserResponse {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

// ============================================================================
// HTTP Client Service
// ============================================================================

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
}

admixture::service! {
    HttpClient {
        error: ClientError,
        client: Client,

        setup {
            base_url: String,
            timeout_secs: u64,
        }

        running {
            client: Client,
            base_url: String,
        }

        async fn start(self) -> Result<HttpClientRunning, ClientError> {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(self.timeout_secs))
                .build()?;

            Ok(HttpClientRunning {
                client,
                base_url: self.base_url,
            })
        }

        async fn client(&self) -> Result<Client, ClientError> {
            Ok(self.client.clone())
        }

        async fn healthy(&self) -> Result<(), ClientError> {
            let response = self.client.get(format!("{}/health", self.base_url)).send().await?;
            if response.status().is_success() {
                Ok(())
            } else {
                Err(ClientError::RequestFailed(response.error_for_status().unwrap_err()))
            }
        }

        async fn stop(&mut self) -> Result<(), ClientError> {
            Ok(())
        }
    }
}

impl HttpClientRunning {
    pub async fn get_health(&self) -> Result<HealthResponse, ClientError> {
        Ok(self.client.get(format!("{}/health", self.base_url))
            .send().await?.json().await?)
    }

    pub async fn get_user(&self, id: u64) -> Result<UserResponse, ClientError> {
        Ok(self.client.get(format!("{}/users/{}", self.base_url, id))
            .send().await?.json().await?)
    }
}

// ============================================================================
// Example Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Axum + Reqwest Integration Testing Example\n");

    // Start server on random port
    let server_setup = AxumServerSetup::construct(AxumServerConfig { port: 0 });
    let server = server_setup.start().await?;
    let base_url = server.client().await?;
    println!("✅ Server started: {}", base_url);

    // Create client
    let client_setup = HttpClientSetup::construct(HttpClientConfig {
        base_url: base_url.clone(),
        timeout_secs: 5,
    });
    let mut client = client_setup.start().await?;
    println!("✅ Client configured\n");

    // Run tests
    println!("Running tests...\n");

    let health = client.get_health().await?;
    println!("✓ Health: {} (v{})", health.status, health.version);

    let user = client.get_user(42).await?;
    println!("✓ User {}: {} <{}>", user.id, user.name, user.email);

    for id in 1..=3 {
        let user = client.get_user(id).await?;
        println!("✓ User {}: {} <{}>", user.id, user.name, user.email);
    }

    // Cleanup
    println!("\n🛑 Shutting down...");
    client.stop().await?;
    println!("✅ Done!");

    Ok(())
}
