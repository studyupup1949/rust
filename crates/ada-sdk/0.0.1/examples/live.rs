use std::env;
use std::sync::Arc;
use std::time::Duration;

use ada_sdk::proto::{BrowserCapability, MintBrowserSessionRequest};
use ada_sdk::{AdaClient, ClientConfig, StreamConfig, SubscriptionOptions};
use tokio::sync::oneshot;

fn required(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("{name} is required"))
}

#[tokio::main]
async fn main() {
    let target = required("ADA_SDK_LIVE_TARGET");
    let endpoint = if target.contains("://") {
        target
    } else {
        format!("http://{target}")
    };
    let api_key = required("ADA_SDK_LIVE_TOKEN");
    let principal_id = required("ADA_SDK_LIVE_PRINCIPAL");
    let document_id = required("ADA_SDK_LIVE_DOCUMENT");
    let client = AdaClient::connect(ClientConfig {
        endpoint,
        api_key,
        insecure: true,
        streams: StreamConfig {
            events: SubscriptionOptions {
                replay_limit: 100,
                max_reconnect_attempts: Some(1),
                ..Default::default()
            },
            ..Default::default()
        },
    })
    .await
    .expect("client connection failed");
    let minted = client
        .mint_browser_session(MintBrowserSessionRequest {
            principal_id: principal_id.clone(),
            capabilities: vec![
                BrowserCapability::Read as i32,
                BrowserCapability::StreamEvents as i32,
            ],
            ttl_seconds: 60,
        })
        .await
        .expect("browser session mint failed");
    assert!(minted.token.starts_with("ada_browser_v1."));

    let (sender, receiver) = oneshot::channel();
    let sender = Arc::new(std::sync::Mutex::new(Some(sender)));
    let expected_document_id = document_id.clone();
    let unsubscribe = client
        .principal(&principal_id)
        .events()
        .on_memory_ingest_started(move |event| {
            if event.document_id == expected_document_id
                && let Some(sender) = sender.lock().expect("sender lock failed").take()
            {
                let _result = sender.send(());
            }
        });
    tokio::time::timeout(Duration::from_secs(10), receiver)
        .await
        .expect("typed event callback timed out")
        .expect("typed event callback channel closed");
    unsubscribe.unsubscribe();
    client.close().await;
}
