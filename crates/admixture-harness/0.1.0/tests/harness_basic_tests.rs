//! Integration tests using admixture_harness

use admixture::context;
use admixture::service;
use admixture::service::ServiceRunning;
use admixture_harness::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Mock service for testing using the service! macro
#[derive(Debug, thiserror::Error)]
#[error("Mock service error")]
pub struct MockServiceError;

service! {
    MockService {
        error: MockServiceError,
        client: String,

        setup {
            name: String,
        }

        running {
            name: String,
            stopped: Arc<AtomicBool>,
        }

        async fn start(self) -> Result<MockServiceRunning, MockServiceError> {
            println!("      🔧 Starting service: {}", self.name);
            Ok(MockServiceRunning {
                name: self.name,
                stopped: Arc::new(AtomicBool::new(false)),
            })
        }

        async fn client(&self) -> Result<String, MockServiceError> {
            Ok(self.name.clone())
        }

        async fn healthy(&self) -> Result<(), MockServiceError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), MockServiceError> {
            println!("      🛑 Stopping service: {}", self.name);
            self.stopped.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
}

// Define test context using the context! macro
context! {
    MockTestContext {
        database: MockServiceSetup = MockServiceConfig { name: "database".to_string() },
        cache: MockServiceSetup = MockServiceConfig { name: "cache".to_string() },
    }
}

// Test functions using the harness
#[admixture_test(context = MockTestContext)]
async fn test_cache_connection(ctx: &MockTestContext) {
    let cache_client = ctx.cache().client().await.unwrap();
    assert_eq!(cache_client, "cache");
}

#[admixture_test(context = MockTestContext)]
async fn test_database_connection(ctx: &MockTestContext) -> Result<(), TestError> {
    let db_client = ctx.database().client().await.unwrap();
    assert_eq!(db_client, "database");
    Ok(())
}

#[admixture_test(context = MockTestContext)]
async fn test_both_services(ctx: &MockTestContext) -> Result<(), TestError> {
    let db_client = ctx.database().client().await.unwrap();
    let cache_client = ctx.cache().client().await.unwrap();
    assert_eq!(db_client, "database");
    assert_eq!(cache_client, "cache");
    Ok(())
}

// Test with custom display name
#[admixture_test(context = MockTestContext, name = "Verify custom test names work correctly")]
async fn test_custom_name(ctx: &MockTestContext) -> Result<(), TestError> {
    // This test demonstrates the custom name feature
    let db_client = ctx.database().client().await.unwrap();
    assert_eq!(db_client, "database");
    Ok(())
}

// Another test with descriptive custom name
#[admixture_test(context = MockTestContext, name = "Test with special characters: ñoño & emojis 🚀")]
async fn test_special_chars(_ctx: &MockTestContext) -> Result<(), TestError> {
    // This test shows that custom names can include special characters
    Ok(())
}

// Generate the test runner
admixture_harness::test_runner!();
