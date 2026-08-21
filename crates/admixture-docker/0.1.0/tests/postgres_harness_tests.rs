//! Integration test with real PostgreSQL using Docker containers
//!
//! This demonstrates the harness with actual testcontainers.

use admixture::context;
use admixture::service::ServiceRunning;
use admixture_docker::SqlxPostgresServiceSetup;
use admixture_harness::prelude::*;
use testcontainers_modules::postgres::Postgres;

// Define the Postgres test context using the context macro
context! {
    PostgresTestContext {
        postgres: SqlxPostgresServiceSetup = Postgres::default(),
    }
}

#[admixture_test(context = PostgresTestContext)]
async fn test_postgres_query(ctx: &PostgresTestContext) -> Result<(), TestError> {
    let client = ctx
        .postgres()
        .client()
        .await
        .map_err(|e| TestError::Other(e.to_string()))?;

    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&client)
        .await
        .map_err(|e| TestError::Other(e.to_string()))?;

    assert_eq!(result, 1, "Query should return 1");

    Ok(())
}

#[admixture_test(context = PostgresTestContext)]
async fn test_postgres_table_creation(ctx: &PostgresTestContext) -> Result<(), TestError> {
    let client = ctx
        .postgres()
        .client()
        .await
        .map_err(|e| TestError::Other(e.to_string()))?;

    sqlx::query("CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT NOT NULL)")
        .execute(&client)
        .await
        .map_err(|e| TestError::Other(e.to_string()))?;

    sqlx::query("INSERT INTO test_table (name) VALUES ($1)")
        .bind("test_name")
        .execute(&client)
        .await
        .map_err(|e| TestError::Other(e.to_string()))?;

    let name: String = sqlx::query_scalar("SELECT name FROM test_table WHERE id = 1")
        .fetch_one(&client)
        .await
        .map_err(|e| TestError::Other(e.to_string()))?;

    assert_eq!(name, "test_name", "Should retrieve the inserted name");

    Ok(())
}

#[admixture_test(context = PostgresTestContext)]
async fn test_postgres_health(ctx: &PostgresTestContext) -> Result<(), TestError> {
    ctx.postgres()
        .healthy()
        .await
        .map_err(|e| TestError::Other(e.to_string()))?;

    Ok(())
}

// Generate the test runner
admixture_harness::test_runner!();
