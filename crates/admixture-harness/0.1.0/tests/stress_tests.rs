//! Stress tests for concurrent context execution.
//!
//! Tests that the framework can handle many concurrent contexts without
//! deadlocks, race conditions, or resource exhaustion.

#![allow(dead_code)]

use admixture::context;
use admixture::service;
use admixture_harness::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// Counter to track concurrent test execution
static CONCURRENT_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, thiserror::Error)]
#[error("Mock service error")]
pub struct MockServiceError;

service! {
    StressTestService {
        error: MockServiceError,

        setup {
            id: u32,
        }

        running {
            id: u32,
            counter: Arc<AtomicU32>,
        }

        async fn start(self) -> Result<StressTestServiceRunning, MockServiceError> {
            // Simulate some startup work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            Ok(StressTestServiceRunning {
                id: self.id,
                counter: Arc::new(AtomicU32::new(0)),
            })
        }

        async fn stop(&mut self) -> Result<(), MockServiceError> {
            // Simulate some cleanup work
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            Ok(())
        }
    }
}

// Define context for stress testing
context! {
    StressTestContext {
        service: StressTestServiceSetup = StressTestServiceConfig { id: 0 },
    }
}

// Generate many tests that will run concurrently
macro_rules! generate_concurrent_tests {
    ($($name:ident: $id:expr),*) => {
        $(
            #[admixture_test(context = StressTestContext)]
            async fn $name(_ctx: &StressTestContext) -> Result<(), TestError> {
                let count = CONCURRENT_COUNTER.fetch_add(1, Ordering::SeqCst);

                // Simulate some test work
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

                // Record that we executed
                println!("      ✓ Test {} executed (concurrent count: {})", $id, count + 1);

                Ok(())
            }
        )*
    };
}

// Generate 50 tests (each context will be started once, so we'll have 50 concurrent contexts)
generate_concurrent_tests!(
    stress_test_01: 1, stress_test_02: 2, stress_test_03: 3, stress_test_04: 4, stress_test_05: 5,
    stress_test_06: 6, stress_test_07: 7, stress_test_08: 8, stress_test_09: 9, stress_test_10: 10,
    stress_test_11: 11, stress_test_12: 12, stress_test_13: 13, stress_test_14: 14, stress_test_15: 15,
    stress_test_16: 16, stress_test_17: 17, stress_test_18: 18, stress_test_19: 19, stress_test_20: 20,
    stress_test_21: 21, stress_test_22: 22, stress_test_23: 23, stress_test_24: 24, stress_test_25: 25,
    stress_test_26: 26, stress_test_27: 27, stress_test_28: 28, stress_test_29: 29, stress_test_30: 30,
    stress_test_31: 31, stress_test_32: 32, stress_test_33: 33, stress_test_34: 34, stress_test_35: 35,
    stress_test_36: 36, stress_test_37: 37, stress_test_38: 38, stress_test_39: 39, stress_test_40: 40,
    stress_test_41: 41, stress_test_42: 42, stress_test_43: 43, stress_test_44: 44, stress_test_45: 45,
    stress_test_46: 46, stress_test_47: 47, stress_test_48: 48, stress_test_49: 49, stress_test_50: 50
);

// Generate the test runner
admixture_harness::test_runner!();
