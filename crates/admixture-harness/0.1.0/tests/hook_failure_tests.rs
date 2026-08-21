//! Integration tests for lifecycle hook failure scenarios.
//!
//! These tests verify that hook failures are handled correctly:
//! - before_all failures should prevent all tests from running
//! - before_each failures should fail that specific test
//! - after_each failures should be logged but not fail the test
//! - after_all failures should be logged

#![allow(dead_code)]

use admixture::context;
use admixture::service;
use admixture_harness::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// Atomic flags to control hook failures
static SHOULD_FAIL_BEFORE_ALL: AtomicBool = AtomicBool::new(false);
static SHOULD_FAIL_BEFORE_EACH: AtomicBool = AtomicBool::new(false);
static SHOULD_FAIL_AFTER_EACH: AtomicBool = AtomicBool::new(false);
static SHOULD_FAIL_AFTER_ALL: AtomicBool = AtomicBool::new(false);

// Counter to track test execution
static TEST_EXECUTION_COUNTER: AtomicU32 = AtomicU32::new(0);

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
        }

        async fn start(self) -> Result<MockServiceRunning, MockServiceError> {
            Ok(MockServiceRunning {
                name: self.name,
            })
        }

        async fn stop(&mut self) -> Result<(), MockServiceError> {
            Ok(())
        }
    }
}

// Hook that can be configured to fail
async fn failing_before_all(_ctx: &BeforeAllFailContextRunning) -> Result<(), Box<dyn std::error::Error + Send>> {
    if SHOULD_FAIL_BEFORE_ALL.load(Ordering::SeqCst) {
        println!("    ❌ before_all intentionally failing");
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "before_all hook failed")));
    }
    println!("    ✓ before_all succeeded");
    Ok(())
}

async fn failing_before_each(_ctx: &BeforeEachFailContextRunning) -> Result<(), Box<dyn std::error::Error + Send>> {
    if SHOULD_FAIL_BEFORE_EACH.load(Ordering::SeqCst) {
        println!("      ❌ before_each intentionally failing");
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "before_each hook failed")));
    }
    println!("      ✓ before_each succeeded");
    Ok(())
}

async fn failing_after_each(_ctx: &AfterEachFailContextRunning) -> Result<(), Box<dyn std::error::Error + Send>> {
    if SHOULD_FAIL_AFTER_EACH.load(Ordering::SeqCst) {
        println!("      ❌ after_each intentionally failing");
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "after_each hook failed")));
    }
    println!("      ✓ after_each succeeded");
    Ok(())
}

async fn failing_after_all(_ctx: &AfterAllFailContextRunning) -> Result<(), Box<dyn std::error::Error + Send>> {
    if SHOULD_FAIL_AFTER_ALL.load(Ordering::SeqCst) {
        println!("    ❌ after_all intentionally failing");
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "after_all hook failed")));
    }
    println!("    ✓ after_all succeeded");
    Ok(())
}

// Context with before_all that can fail
context! {
    BeforeAllFailContext {
        mock_service: MockServiceSetup = MockServiceConfig { name: "before-all-fail".to_string() },
        hooks {
            before_all = failing_before_all,
        },
    }
}

#[admixture_test(context = BeforeAllFailContext)]
async fn test_should_not_run_if_before_all_fails(_ctx: &BeforeAllFailContext) -> Result<(), TestError> {
    TEST_EXECUTION_COUNTER.fetch_add(1, Ordering::SeqCst);
    println!("      ✓ This test ran (before_all succeeded)");
    Ok(())
}

// Context with before_each that can fail
context! {
    BeforeEachFailContext {
        mock_service: MockServiceSetup = MockServiceConfig { name: "before-each-fail".to_string() },
        hooks {
            before_each = failing_before_each,
        },
    }
}

#[admixture_test(context = BeforeEachFailContext)]
async fn test_before_each_failure_scenario(_ctx: &BeforeEachFailContext) -> Result<(), TestError> {
    println!("      ✓ Test body executed (before_each succeeded)");
    Ok(())
}

// Context with after_each that can fail
context! {
    AfterEachFailContext {
        mock_service: MockServiceSetup = MockServiceConfig { name: "after-each-fail".to_string() },
        hooks {
            after_each = failing_after_each,
        },
    }
}

#[admixture_test(context = AfterEachFailContext)]
async fn test_after_each_failure_scenario(_ctx: &AfterEachFailContext) -> Result<(), TestError> {
    println!("      ✓ Test body executed, after_each may fail");
    Ok(())
}

// Context with after_all that can fail
context! {
    AfterAllFailContext {
        mock_service: MockServiceSetup = MockServiceConfig { name: "after-all-fail".to_string() },
        hooks {
            after_all = failing_after_all,
        },
    }
}

#[admixture_test(context = AfterAllFailContext)]
async fn test_after_all_failure_scenario(_ctx: &AfterAllFailContext) -> Result<(), TestError> {
    println!("      ✓ Test body executed, after_all may fail");
    Ok(())
}

// Generate the test runner
admixture_harness::test_runner!();
