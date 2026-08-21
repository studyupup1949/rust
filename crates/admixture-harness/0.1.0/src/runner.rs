//! Test runner that executes all registered integration tests.

use crate::{ContextManager, TestDescriptor};
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Result of running all tests.
#[derive(Debug)]
pub struct TestResults {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<TestFailure>,
}

/// Information about a failed test.
#[derive(Debug, Clone)]
pub struct TestFailure {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub error: String,
}

impl TestFailure {
    /// Create a test failure for a context start failure.
    fn context_start_failed(test: &TestDescriptor, error: String) -> Self {
        Self {
            name: test.name.to_string(),
            file: test.file.to_string(),
            line: test.line,
            error: format!("Context startup failed: {}", error),
        }
    }

    /// Create a test failure for a hook failure.
    fn hook_failed(test: &TestDescriptor, hook_name: &str, error: String) -> Self {
        Self {
            name: test.name.to_string(),
            file: test.file.to_string(),
            line: test.line,
            error: format!("Hook '{}' failed: {}", hook_name, error),
        }
    }

    /// Create a test failure for a test failure.
    fn test_failed(test: &TestDescriptor, error: String) -> Self {
        Self {
            name: test.name.to_string(),
            file: test.file.to_string(),
            line: test.line,
            error,
        }
    }
}

impl TestResults {
    /// Check if all tests passed.
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Print a summary of test results.
    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(80));
        println!("📊 Test Results Summary");
        println!("{}", "=".repeat(80));

        if self.failed == 0 {
            println!("   ✅ All tests passed!");
        } else {
            println!("   ⚠️  Some tests failed");
        }

        println!();
        println!("   Total:  {}", self.total);
        println!("   ✅ Passed: {}", self.passed);

        if !self.failures.is_empty() {
            println!("   ❌ Failed: {}", self.failed);
            println!("\n{}", "-".repeat(80));
            println!("❌ Failed Tests:");
            println!("{}", "-".repeat(80));
            for failure in &self.failures {
                println!("\n   Test: {}", failure.name);
                println!("   Location: {}:{}", failure.file, failure.line);
                println!("   Error: {}", failure.error);
            }
        }

        println!("{}", "=".repeat(80));
    }
}

/// Collect all registered integration tests.
pub fn collect_tests() -> Vec<&'static TestDescriptor> {
    inventory::iter::<TestDescriptor>().collect()
}

/// Run all registered integration tests, grouped by context type.
///
/// Tests sharing the same context type will reuse the same context instance.
/// This function initializes basic tracing for output.
#[tracing::instrument(name = "test_run", skip_all, fields(total_tests, context_types))]
pub async fn run_all_tests() -> TestResults {
    // Initialize basic tracing if not already initialized
    let _ = tracing_subscriber::fmt()
        .with_target(false)
        .with_test_writer()
        .try_init();

    let tests = collect_tests();

    tracing::Span::current().record("total_tests", tests.len());
    info!("Starting integration test run");

    // Group tests by context type
    let mut grouped_tests: HashMap<&str, Vec<&TestDescriptor>> = HashMap::new();
    for test in tests.iter() {
        grouped_tests
            .entry(test.context_type)
            .or_default()
            .push(test);
    }

    tracing::Span::current().record("context_types", grouped_tests.len());
    info!("Grouped tests by context type");

    run_all_tests_impl(tests, grouped_tests).await
}

/// Internal implementation of test runner.
///
/// This is exposed as a public API so that external tools (like admixture-tui)
/// can run tests with custom tracing layers.
pub async fn run_all_tests_impl(
    tests: Vec<&'static TestDescriptor>,
    grouped_tests: HashMap<&'static str, Vec<&'static TestDescriptor>>,
) -> TestResults {
    info!("Starting integration test run");

    // Run all context groups in parallel
    let futures: Vec<_> = grouped_tests
        .into_values()
        .map(|group| {
            let group_slice: &'static [&'static TestDescriptor] = Box::leak(group.into_boxed_slice());
            async move {
                // Call the type-erased entry point for this context type
                // Internally calls run_context_group<ConcreteType>
                (group_slice[0].run_group)(group_slice).await
            }
        })
        .collect();

    let group_results = futures::future::join_all(futures).await;

    // Aggregate results
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    for group_result in group_results {
        passed += group_result.passed;
        failed += group_result.failed;
        failures.extend(group_result.failures);
    }

    let results = TestResults {
        total: tests.len(),
        passed,
        failed,
        failures,
    };

    info!(
        total = results.total,
        passed = results.passed,
        failed = results.failed,
        "Test run completed"
    );

    results.print_summary();

    results
}

/// Result from running a context group.
///
/// This is public so that it can be used in GroupRunnerFn.
pub struct ContextGroupResult {
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<TestFailure>,
}

/// Run all tests for a single context type with concrete type C.
///
/// This is the generic execution layer - all tests in the group share the same
/// context type C, so no type erasure or downcasting is needed.
#[tracing::instrument(
    name = "context_group",
    skip_all,
    fields(
        context_type = %context_type,
        test_count = tests.len(),
        status = tracing::field::Empty,
    )
)]
pub async fn run_context_group<C: Send + 'static>(
    context_type: &str,
    tests: &[(crate::TestFn<C>, &'static TestDescriptor)],
    manager: &dyn ContextManager<C>,
    hooks: crate::Hooks<C>,
) -> ContextGroupResult {
    info!(
        context_type = context_type,
        test_count = tests.len(),
        "CONTEXT_GROUP_START"
    );

    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    // Start the context once for all tests - concrete type C!
    tracing::Span::current().record("status", "starting context");
    let ctx = match manager.start().await {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::Span::current().record("status", "❌ failed to start");
            let error_msg = e.to_string();
            error!(error = %e, "Failed to start context");

            // Mark all tests in this group as failed
            for &(_, descriptor) in tests {
                failures.push(TestFailure::context_start_failed(descriptor, error_msg.clone()));
            }

            return ContextGroupResult {
                passed: 0,
                failed: tests.len(),
                failures,
            };
        }
    };

    tracing::Span::current().record("status", "✅ context ready");

    // Run before_all hook - direct call, no downcast!
    if let Some(before_all_fn) = hooks.before_all
        && let Err(e) = before_all_fn(&ctx).await {
        // before_all failed - fail all tests and stop
        error!(
            "before_all hook failed, skipping all tests in context {}",
            context_type
        );
        tracing::Span::current().record("status", "❌ before_all failed");

        for &(_, descriptor) in tests {
            failures.push(TestFailure::hook_failed(
                descriptor,
                "before_all",
                e.to_string(),
            ));
        }

        // Best-effort context stop
        if let Err(stop_err) = manager.stop(ctx).await {
            error!(error = %stop_err, "Failed to stop context after before_all failure");
        }

        return ContextGroupResult {
            passed: 0,
            failed: tests.len(),
            failures,
        };
    }

    // Run all tests with the shared context
    for &(test_fn, descriptor) in tests {
        // Run before_each hook - direct call, no downcast!
        if let Some(before_each_fn) = hooks.before_each
            && let Err(e) = before_each_fn(&ctx).await {
            failed += 1;
            failures.push(TestFailure::hook_failed(
                descriptor,
                "before_each",
                e.to_string(),
            ));

            // Run after_each even though test didn't run
            if let Some(after_each_fn) = hooks.after_each {
                let _ = after_each_fn(&ctx).await;
            }

            continue; // Skip to next test
        }

        // Run the test - direct call, no downcast!
        let test_result = run_test(descriptor, test_fn, &ctx).await;

        // Run after_each hook (always runs) - direct call, no downcast!
        if let Some(after_each_fn) = hooks.after_each
            && let Err(e) = after_each_fn(&ctx).await {
            // after_each failed - mark test as failed
            failed += 1;
            failures.push(TestFailure::hook_failed(
                descriptor,
                "after_each",
                e.to_string(),
            ));
            continue;
        }

        // Record test result
        match test_result {
            Ok(()) => {
                passed += 1;
            }
            Err(e) => {
                failed += 1;
                failures.push(TestFailure::test_failed(descriptor, e.to_string()));
            }
        }
    }

    // Run after_all hook (best-effort, doesn't fail tests) - direct call, no downcast!
    if let Some(after_all_fn) = hooks.after_all
        && let Err(e) = after_all_fn(&ctx).await {
        tracing::warn!("after_all hook failed (non-fatal): {}", e);
    }

    // Stop the context after all tests complete - concrete type C!
    tracing::Span::current().record("status", "🛑 stopping");
    if let Err(e) = manager.stop(ctx).await {
        error!(error = %e, "Failed to stop context");
        tracing::Span::current().record("status", "⚠️ completed with stop error");
    } else {
        tracing::Span::current().record("status", "✅ completed");
    }

    info!("CONTEXT_GROUP_END");

    ContextGroupResult {
        passed,
        failed,
        failures,
    }
}

/// Run a single test with concrete context type - no downcast needed!
#[tracing::instrument(
    name = "test",
    skip_all,
    fields(
        name = %descriptor.name,
        context_type = %descriptor.context_type,
        file = %descriptor.file,
        line = descriptor.line,
        result = tracing::field::Empty,
    )
)]
async fn run_test<C>(
    descriptor: &TestDescriptor,
    test_fn: crate::TestFn<C>,
    ctx: &C,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    debug!("Running test");

    match test_fn(ctx).await {
        Ok(()) => {
            tracing::Span::current().record("result", "✅ passed");
            info!(name = descriptor.name, result = "✅ passed", "TEST_PASSED");
            Ok(())
        }
        Err(e) => {
            tracing::Span::current().record("result", "❌ failed");
            error!(name = descriptor.name, error = %e, "TEST_FAILED");
            Err(e)
        }
    }
}

/// Macro to generate a test runner function.
///
/// Use this in your test module to create a single test that runs all
/// `#[admixture_test]` tests. Automatically detects if TUI mode is enabled
/// via the ADMIXTURE_TUI environment variable.
///
/// # Example
///
/// ```ignore
/// use admixture_harness::test_runner;
///
/// test_runner!();
/// ```
#[macro_export]
macro_rules! test_runner {
    () => {
        #[tokio::test]
        async fn __run_all_admixture_tests() {
            let results = $crate::runner::run_all_tests().await;
            assert!(results.all_passed(), "{} test(s) failed", results.failed);
        }
    };
}
