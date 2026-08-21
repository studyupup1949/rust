//! Procedural macros for Admixture test framework.
//!
//! This crate provides macros for declaratively defining test contexts and services.

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod context;
mod service;

/// Declares a test context with multiple services.
///
/// Supports optional inline configuration for each service using the syntax:
/// `service_name: ServiceType = config_expr`
///
/// # Examples
///
/// ## With inline configuration (recommended for different configs)
///
/// ```ignore
/// use admixture::context;
/// use testcontainers_modules::postgres::Postgres;
///
/// context! {
///     MyTestContext {
///         primary_db: SqlxPostgresServiceSetup = Postgres::default().with_tag("15"),
///         replica_db: SqlxPostgresServiceSetup = Postgres::default().with_tag("14"),
///         redis: RedisServiceSetup = Redis::default(),
///     }
/// }
/// ```
///
/// ## Without inline configuration (uses Config::default())
///
/// ```ignore
/// use admixture::context;
///
/// context! {
///     MyTestContext {
///         postgres: SqlxPostgresServiceSetup,
///         redis: RedisServiceSetup,
///     }
/// }
/// // Requires that each service's Config type implements Default
/// ```
///
/// ## Lifecycle Hooks
///
/// Contexts can define optional lifecycle hooks that run at specific points
/// in the test lifecycle. Hooks are defined in a separate `hooks { ... }` block
/// and can be placed in any order relative to service definitions.
///
/// ```ignore
/// use admixture::context;
/// use std::error::Error;
///
/// context! {
///     TestContext {
///         postgres: PostgresServiceSetup,
///     },
///     hooks {
///         before_all = setup_test_data,
///         after_all = cleanup_test_data,
///         before_each = reset_database,
///         after_each = verify_invariants,
///     }
/// }
///
/// async fn setup_test_data(ctx: &TestContextRunning) -> Result<(), Box<dyn Error + Send>> {
///     // Runs once after context starts, before any tests
///     let client = ctx.postgres().client().await?;
///     client.execute("INSERT INTO users (name) VALUES ('test')", &[]).await?;
///     Ok(())
/// }
///
/// async fn reset_database(ctx: &TestContextRunning) -> Result<(), Box<dyn Error + Send>> {
///     // Runs before each test
///     ctx.postgres().client().await?.execute("TRUNCATE users", &[]).await?;
///     Ok(())
/// }
///
/// async fn verify_invariants(ctx: &TestContextRunning) -> Result<(), Box<dyn Error + Send>> {
///     // Runs after each test (even if test failed)
///     let count: i64 = ctx.postgres().client().await?
///         .query_one("SELECT COUNT(*) FROM users WHERE invalid = true", &[])
///         .await?
///         .get(0);
///     if count > 0 {
///         return Err("Found invalid users after test".into());
///     }
///     Ok(())
/// }
///
/// async fn cleanup_test_data(ctx: &TestContextRunning) -> Result<(), Box<dyn Error + Send>> {
///     // Runs once after all tests, before context stops (best-effort)
///     ctx.postgres().client().await?.execute("DROP TABLE IF EXISTS temp_data", &[]).await?;
///     Ok(())
/// }
/// ```
///
/// ### Hook Execution Order
///
/// 1. Context starts
/// 2. **before_all** - If this fails, all tests in the context group fail
/// 3. For each test:
///    - **before_each** - If this fails, that specific test fails
///    - Test runs
///    - **after_each** - If this fails, that test fails (always runs, even if test failed)
/// 4. **after_all** - Failure is logged but doesn't fail tests (best-effort cleanup)
/// 5. Context stops
///
/// ### Hook Failure Behavior
///
/// - `before_all` failure → all tests in context group fail
/// - `before_each` failure → that specific test fails
/// - `after_each` failure → that specific test fails  
/// - `after_all` failure → logged as warning, doesn't fail tests
///
/// ### Hook Function Signature
///
/// All hook functions must have this signature:
/// ```ignore
/// async fn hook_name(ctx: &ContextNameRunning) -> Result<(), Box<dyn Error + Send>>
/// ```
///
/// ### Flexible Ordering
///
/// Both services and hooks can appear in any order:
///
/// ```ignore
/// context! {
///     MyContext {
///         hooks {
///             before_each = reset_state,
///         },
///         postgres: PostgresServiceSetup,
///         hooks {  // Can even split hooks if needed (though not recommended)
///             after_each = verify_state,
///         },
///         redis: RedisServiceSetup,
///     }
/// }
/// ```
///
/// This generates:
/// - A `MyTestContextConfig` struct with service config fields
/// - A `MyTestContextSetup` struct with service setup fields
/// - A `MyTestContextRunning` struct with running service fields
/// - Implementations of `ContextSetup` and `ContextRunning` traits
/// - A type alias `MyTestContext = TestContext<MyTestContextRunning>`
/// - A constructor method `MyTestContext::new(setup)`
/// - A static `MYTESTCONTEXT_HOOKS` containing hook function pointers
#[proc_macro]
pub fn context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as context::ContextMacroInput);
    context::generate(input).into()
}

/// Declares a service with setup and running states.
///
/// ## Flexible Ordering
///
/// Elements can be specified in **any order** (except the service name must be first).
/// This allows you to organize your service definition in the way that makes most sense:
///
/// ```ignore
/// // Functions first
/// service! {
///     MyService {
///         error: MyError,
///         
///         async fn start(self) -> Result<MyServiceRunning, MyError> { /**/ }
///         async fn stop(&mut self) -> Result<(), MyError> { /**/ }
///         
///         running { connection: Connection, }
///     }
/// }
///
/// // Or fields first
/// service! {
///     MyService {
///         error: MyError,
///         
///         setup { config: Config, }
///         running { handle: Handle, }
///         
///         async fn start(self) -> Result<MyServiceRunning, MyError> { /**/ }
///         async fn stop(&mut self) -> Result<(), MyError> { /**/ }
///     }
/// }
/// ```
///
/// ## Optional Elements
///
/// All of `setup`, `running`, `client`, and `healthy` are optional:
/// - `setup` defaults to an empty struct if omitted
/// - `running` defaults to an empty struct if omitted
/// - `client` defaults to `()` with a default implementation returning `Ok(())`
/// - `healthy` defaults to a no-op implementation returning `Ok(())`
///
/// ## Required Elements
///
/// Only three elements are required:
/// - `error: ErrorType,` - The error type for this service
/// - `async fn start(...)` - Transition from setup to running state
/// - `async fn stop(...)` - Cleanup when stopping the service
///
/// # Absolute minimal example (no setup/running fields, no client)
///
/// ```ignore
/// use admixture::service;
///
/// service! {
///     TimerService {
///         error: TimerError,
///         
///         async fn start(self) -> Result<TimerServiceRunning, TimerError> {
///             tokio::spawn(async { /* background timer */ });
///             Ok(TimerServiceRunning {})
///         }
///         
///         async fn stop(&mut self) -> Result<(), TimerError> {
///             Ok(())
///         }
///     }
/// }
/// // No setup, running, client, or healthy needed!
/// // All auto-generated with sensible defaults
/// ```
///
/// # Example with client and custom health check
///
/// ```ignore
/// use admixture::service;
///
/// service! {
///     MockService {
///         error: MockError,
///         client: String,
///         
///         setup {
///             name: String,
///         }
///         
///         running {
///             name: String,
///         }
///         
///         async fn start(self) -> Result<MockServiceRunning, MockError> {
///             Ok(MockServiceRunning { name: self.name })
///         }
///         
///         async fn client(&self) -> Result<String, MockError> {
///             Ok(self.name.clone())
///         }
///         
///         async fn healthy(&self) -> Result<(), MockError> {
///             // Custom health check logic
///             if self.name.is_empty() {
///                 Err(MockError::InvalidState)
///             } else {
///                 Ok(())
///             }
///         }
///         
///         async fn stop(&mut self) -> Result<(), MockError> {
///             Ok(())
///         }
///     }
/// }
/// ```
///
/// # Example with flexible ordering
///
/// ```ignore
/// use admixture::service;
///
/// service! {
///     BackgroundService {
///         error: BackgroundError,
///         
///         // Stop function first
///         async fn stop(&mut self) -> Result<(), BackgroundError> {
///             self.handle.abort();
///             Ok(())
///         }
///         
///         // Running state in the middle
///         running {
///             handle: JoinHandle<()>,
///         }
///         
///         // Start function last
///         async fn start(self) -> Result<BackgroundServiceRunning, BackgroundError> {
///             let handle = tokio::spawn(async { /* background work */ });
///             Ok(BackgroundServiceRunning { handle })
///         }
///     }
/// }
/// // Order doesn't matter - organize however makes sense!
/// ```
///
/// This generates:
/// - A `MockServiceConfig` struct matching setup fields
/// - A `MockServiceSetup` struct with the setup fields
/// - A `MockServiceRunning` struct with the running fields
/// - Implementations of `ServiceSetup` and `ServiceRunning` traits
#[proc_macro]
pub fn service(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as service::ServiceMacroInput);
    service::generate(input).into()
}
