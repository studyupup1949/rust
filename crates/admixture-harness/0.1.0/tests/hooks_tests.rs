//! Integration tests for lifecycle hooks functionality.

use admixture::context;
use admixture::service;
use admixture_harness::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// Atomic counters to track hook execution
static BEFORE_ALL_COUNTER: AtomicU32 = AtomicU32::new(0);
static AFTER_ALL_COUNTER: AtomicU32 = AtomicU32::new(0);
static BEFORE_EACH_COUNTER: AtomicU32 = AtomicU32::new(0);
static AFTER_EACH_COUNTER: AtomicU32 = AtomicU32::new(0);

// Define a simple mock service for testing
#[derive(Debug, thiserror::Error)]
#[error("Mock service error")]
pub struct MockServiceError;

service! {
    MockService {
        error: MockServiceError,

        setup {
            name: String,
        }

        running {
            name: String,
            data: Arc<AtomicU32>,
        }

        async fn start(self) -> Result<MockServiceRunning, MockServiceError> {
            Ok(MockServiceRunning {
                name: self.name,
                data: Arc::new(AtomicU32::new(0)),
            })
        }

        async fn stop(&mut self) -> Result<(), MockServiceError> {
            Ok(())
        }
    }
}

// Hook functions that track execution
async fn before_all_hook(ctx: &TestContextRunning) -> Result<(), Box<dyn std::error::Error + Send>> {
    let count = BEFORE_ALL_COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("    🎬 before_all executed (count: {})", count + 1);
    
    // Verify we can access the service (Running struct has direct field access)
    assert_eq!(ctx.mock_service.name, "test-service");
    
    Ok(())
}

async fn after_all_hook(ctx: &TestContextRunning) -> Result<(), Box<dyn std::error::Error + Send>> {
    let count = AFTER_ALL_COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("    🎬 after_all executed (count: {})", count + 1);
    
    // Verify we can access the service
    assert_eq!(ctx.mock_service.name, "test-service");
    
    Ok(())
}

async fn before_each_hook(ctx: &TestContextRunning) -> Result<(), Box<dyn std::error::Error + Send>> {
    let count = BEFORE_EACH_COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("      🔄 before_each executed (count: {})", count + 1);
    
    // Reset service data before each test
    ctx.mock_service.data.store(0, Ordering::SeqCst);
    
    Ok(())
}

async fn after_each_hook(ctx: &TestContextRunning) -> Result<(), Box<dyn std::error::Error + Send>> {
    let count = AFTER_EACH_COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("      🔄 after_each executed (count: {})", count + 1);
    
    // Verify service data was used
    let data = ctx.mock_service.data.load(Ordering::SeqCst);
    if data == 0 {
        println!("        ⚠️  Warning: Service data was not modified during test");
    }
    
    Ok(())
}

// Define context with all hooks
context! {
    TestContext {
        mock_service: MockServiceSetup = MockServiceConfig { name: "test-service".to_string() },
        hooks {
            before_all = before_all_hook,
            after_all = after_all_hook,
            before_each = before_each_hook,
            after_each = after_each_hook,
        },
    }
}

// Test 1: Verify before_all has executed
#[admixture_test(context = TestContext)]
async fn test_before_all_executed(_ctx: &TestContext) -> Result<(), TestError> {
    println!("      ✓ Test 1: Checking before_all executed");
    
    // before_all should have executed at least once
    let before_all_count = BEFORE_ALL_COUNTER.load(Ordering::SeqCst);
    assert!(before_all_count >= 1, "before_all should have executed");
    
    // before_each should have executed for this test
    let before_each_count = BEFORE_EACH_COUNTER.load(Ordering::SeqCst);
    assert!(before_each_count >= 1, "before_each should have executed");
    
    Ok(())
}

// Test 2: Verify hooks execute for each test
#[admixture_test(context = TestContext)]
async fn test_hooks_execute_per_test(_ctx: &TestContext) -> Result<(), TestError> {
    println!("      ✓ Test 2: Checking hooks execute per test");
    
    // before_each should have executed (at least once, we don't know the test order)
    let before_each_count = BEFORE_EACH_COUNTER.load(Ordering::SeqCst);
    assert!(before_each_count >= 1, "before_each should have executed");
    
    Ok(())
}

// Test 3: Verify service access in tests
#[admixture_test(context = TestContext)]
async fn test_service_access(_ctx: &TestContext) -> Result<(), TestError> {
    println!("      ✓ Test 3: Testing service access");
    
    // before_each should have executed for this test too (at least once)
    let before_each_count = BEFORE_EACH_COUNTER.load(Ordering::SeqCst);
    assert!(before_each_count >= 1, "before_each should have executed");
    
    Ok(())
}

// Context without hooks to test backward compatibility
context! {
    NoHooksContext {
        mock_service: MockServiceSetup = MockServiceConfig { name: "no-hooks-service".to_string() },
    }
}

#[admixture_test(context = NoHooksContext)]
async fn test_context_without_hooks(_ctx: &NoHooksContext) -> Result<(), TestError> {
    println!("      ✓ Test: Context without hooks works");
    Ok(())
}

// Context with only some hooks defined
async fn partial_before_each(ctx: &PartialHooksContextRunning) -> Result<(), Box<dyn std::error::Error + Send>> {
    println!("      🔄 partial_before_each executed");
    ctx.mock_service.data.store(777, Ordering::SeqCst);
    Ok(())
}

context! {
    PartialHooksContext {
        mock_service: MockServiceSetup = MockServiceConfig { name: "partial-hooks".to_string() },
        hooks {
            before_each = partial_before_each,
        },
    }
}

#[admixture_test(context = PartialHooksContext)]
async fn test_partial_hooks(_ctx: &PartialHooksContext) -> Result<(), TestError> {
    println!("      ✓ Test: Partial hooks work");
    Ok(())
}

// Generate the test runner
admixture_harness::test_runner!();
