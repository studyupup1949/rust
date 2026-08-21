//! Test demonstrating multiple context types and context reuse
//!
//! This file shows how the harness groups tests by context type and reuses contexts.

use admixture::context;
use admixture::service;
use admixture::service::ServiceRunning;
use admixture_harness::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// First service type using service! macro
#[derive(Debug, thiserror::Error)]
#[error("Database service error")]
pub struct DatabaseServiceError;

service! {
    DatabaseService {
        error: DatabaseServiceError,

        running {
            stopped: Arc<AtomicBool>,
        }

        async fn start(self) -> Result<DatabaseServiceRunning, DatabaseServiceError> {
            println!("      🔧 Starting Database service");
            Ok(DatabaseServiceRunning {
                stopped: Arc::new(AtomicBool::new(false)),
            })
        }

        async fn stop(&mut self) -> Result<(), DatabaseServiceError> {
            println!("      🛑 Stopping Database service");
            self.stopped.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
}

// Second service type using service! macro
#[derive(Debug, thiserror::Error)]
#[error("Cache service error")]
pub struct CacheServiceError;

service! {
    CacheService {
        error: CacheServiceError,

        running {
            stopped: Arc<AtomicBool>,
        }

        async fn start(self) -> Result<CacheServiceRunning, CacheServiceError> {
            println!("      🔧 Starting Cache service");
            Ok(CacheServiceRunning {
                stopped: Arc::new(AtomicBool::new(false)),
            })
        }

        async fn stop(&mut self) -> Result<(), CacheServiceError> {
            println!("      🛑 Stopping Cache service");
            self.stopped.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
}

// Define first context with just database
context! {
    DatabaseContext {
        database: DatabaseServiceSetup = DatabaseServiceConfig {},
    }
}

// Define second context with just cache
context! {
    CacheContext {
        cache: CacheServiceSetup = CacheServiceConfig {},
    }
}

// Define third context with both services
context! {
    FullContext {
        database: DatabaseServiceSetup = DatabaseServiceConfig {},
        cache: CacheServiceSetup = CacheServiceConfig {},
    }
}

// Tests using DatabaseContext (will share context)
#[admixture_test(context = DatabaseContext)]
async fn test_database_query_1(ctx: &DatabaseContext) -> Result<(), TestError> {
    ctx.database().healthy().await.unwrap();
    Ok(())
}

#[admixture_test(context = DatabaseContext)]
async fn test_database_query_2(ctx: &DatabaseContext) -> Result<(), TestError> {
    ctx.database().healthy().await.unwrap();
    Ok(())
}

// Tests using CacheContext (will share context)
#[admixture_test(context = CacheContext)]
async fn test_cache_get_1(ctx: &CacheContext) -> Result<(), TestError> {
    ctx.cache().healthy().await.unwrap();
    Ok(())
}

#[admixture_test(context = CacheContext)]
async fn test_cache_get_2(ctx: &CacheContext) -> Result<(), TestError> {
    ctx.cache().healthy().await.unwrap();
    Ok(())
}

// Tests using FullContext (will share context)
#[admixture_test(context = FullContext)]
async fn test_full_integration_1(ctx: &FullContext) -> Result<(), TestError> {
    ctx.database().healthy().await.unwrap();
    ctx.cache().healthy().await.unwrap();
    Ok(())
}

#[admixture_test(context = FullContext)]
async fn test_full_integration_2(ctx: &FullContext) -> Result<(), TestError> {
    ctx.database().healthy().await.unwrap();
    ctx.cache().healthy().await.unwrap();
    Ok(())
}

#[admixture_test(context = FullContext)]
async fn test_full_integration_3(ctx: &FullContext) -> Result<(), TestError> {
    ctx.database().healthy().await.unwrap();
    ctx.cache().healthy().await.unwrap();
    Ok(())
}

// Generate the test runner
admixture_harness::test_runner!();
