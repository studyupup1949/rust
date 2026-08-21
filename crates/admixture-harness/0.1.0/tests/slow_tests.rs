//! Slow-running tests to verify progress indicators and structured logging
//!
//! These tests have deliberate delays to ensure:
//! - Progress bars are visible during execution
//! - Tracing spans show proper hierarchy
//! - Context lifecycle timing is captured

use admixture::context;
use admixture::service;
use admixture::service::ServiceRunning;
use admixture_harness::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

// Slow service that takes time to start and stop using service! macro
#[derive(Debug, thiserror::Error)]
#[error("Slow service error")]
pub struct SlowServiceError;

service! {
    SlowService {
        error: SlowServiceError,

        setup {
            setup_duration_ms: u64,
        }

        running {
            setup_duration_ms: u64,
        }

        async fn start(self) -> Result<SlowServiceRunning, SlowServiceError> {
            tracing::info!(
                "      🔧 Starting slow service ({}ms)...",
                self.setup_duration_ms
            );
            sleep(Duration::from_millis(self.setup_duration_ms)).await;
            tracing::info!("      ✅ Slow service started!");

            Ok(SlowServiceRunning {
                setup_duration_ms: self.setup_duration_ms,
            })
        }

        async fn stop(&mut self) -> Result<(), SlowServiceError> {
            let teardown_duration = self.setup_duration_ms / 2;
            tracing::info!(
                "      🛑 Stopping slow service ({}ms)...",
                teardown_duration
            );
            sleep(Duration::from_millis(teardown_duration)).await;
            tracing::info!("      ✅ Slow service stopped!");
            Ok(())
        }
    }
}

// Very slow service (5 seconds setup) using service! macro
#[derive(Debug, thiserror::Error)]
#[error("Very slow service error")]
pub struct VerySlowServiceError;

service! {
    VerySlowService {
        error: VerySlowServiceError,

        async fn start(self) -> Result<VerySlowServiceRunning, VerySlowServiceError> {
            tracing::info!("      🔧 Starting VERY slow service (5000ms)...");
            for i in 1..=5 {
                sleep(Duration::from_millis(1000)).await;
                tracing::info!("      📊 Setup progress: {}s / 5s", i);
            }
            tracing::info!("      ✅ Very slow service started!");

            Ok(VerySlowServiceRunning {})
        }

        async fn stop(&mut self) -> Result<(), VerySlowServiceError> {
            tracing::info!("      🛑 Stopping very slow service (2500ms)...");
            for i in 1..=5 {
                sleep(Duration::from_millis(500)).await;
                tracing::info!("      📊 Teardown progress: {}%", i * 20);
            }
            tracing::info!("      ✅ Very slow service stopped!");
            Ok(())
        }
    }
}

// Define contexts using the context! macro
context! {
    SlowContext {
        slow: SlowServiceSetup = SlowServiceConfig { setup_duration_ms: 2000 },
    }
}

context! {
    VerySlowContext {
        very_slow: VerySlowServiceSetup = VerySlowServiceConfig {},
    }
}

// Test 1: Quick test with slow context
#[admixture_test(context = SlowContext)]
async fn test_quick_operation(ctx: &SlowContext) -> Result<(), TestError> {
    tracing::info!("      ⚡ Running quick test");
    // Service is already running, just use it
    ctx.slow().healthy().await.unwrap();
    sleep(Duration::from_millis(100)).await;
    Ok(())
}

// Test 2: Medium duration test
#[admixture_test(context = SlowContext)]
async fn test_medium_operation(ctx: &SlowContext) -> Result<(), TestError> {
    tracing::info!("      ⏱️  Running medium test");
    ctx.slow().healthy().await.unwrap();
    for i in 1..=3 {
        sleep(Duration::from_millis(500)).await;
        tracing::info!("      📊 Medium test progress: step {}/3", i);
    }
    Ok(())
}

// Test 3: Slow test with slow context
#[admixture_test(context = SlowContext)]
async fn test_slow_operation(ctx: &SlowContext) -> Result<(), TestError> {
    tracing::info!("      🐌 Running slow test");
    ctx.slow().healthy().await.unwrap();
    for i in 1..=5 {
        sleep(Duration::from_millis(600)).await;
        tracing::info!("      📊 Slow test progress: {:.1}s / 3.0s", i as f32 * 0.6);
    }
    Ok(())
}

// Test 4: Very quick test on very slow context
#[admixture_test(context = VerySlowContext)]
async fn test_instant_on_very_slow_context(ctx: &VerySlowContext) -> Result<(), TestError> {
    tracing::info!("      ⚡ Instant test");
    ctx.very_slow().healthy().await.unwrap();
    Ok(())
}

// Test 5: Another quick test to show context reuse
#[admixture_test(context = VerySlowContext)]
async fn test_another_instant(ctx: &VerySlowContext) -> Result<(), TestError> {
    tracing::info!("      ⚡ Another instant test");
    ctx.very_slow().healthy().await.unwrap();
    sleep(Duration::from_millis(200)).await;
    Ok(())
}

// Test 6: Simulate multi-step process
#[admixture_test(context = VerySlowContext)]
async fn test_multi_step_process(ctx: &VerySlowContext) -> Result<(), TestError> {
    tracing::info!("      🔄 Starting multi-step process");
    ctx.very_slow().healthy().await.unwrap();

    tracing::info!("      📝 Step 1: Preparing data...");
    sleep(Duration::from_millis(800)).await;

    tracing::info!("      ⚙️  Step 2: Processing data...");
    sleep(Duration::from_millis(1200)).await;

    tracing::info!("      ✓  Step 3: Validating results...");
    sleep(Duration::from_millis(600)).await;

    tracing::info!("      ✅ Multi-step process complete!");
    Ok(())
}

admixture_harness::test_runner!();
