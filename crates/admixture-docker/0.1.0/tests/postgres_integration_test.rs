//! Integration tests using real Docker containers with the Context system.

#![cfg(feature = "sqlx-postgres")]

use admixture::context;
use admixture::context::ContextSetup;
use admixture::service::ServiceRunning;
use admixture_docker::SqlxPostgresServiceSetup;
use testcontainers_modules::postgres::Postgres;

// Define the Postgres test context using the macro
context! {
    PostgresContext {
        postgres: SqlxPostgresServiceSetup = Postgres::default(),
    }
}

#[tokio::test]
async fn test_postgres_context_lifecycle() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let config = PostgresContextConfig {
        postgres: Postgres::default(),
    };
    let setup = PostgresContextSetup::construct(config);
    let ctx = PostgresContext::new(setup).build().await?;

    let client = ctx.postgres().client().await?;

    let result: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&client).await?;

    assert_eq!(result, 1, "Query should return 1");

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_postgres_context_with_table_operations() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let config = PostgresContextConfig {
        postgres: Postgres::default(),
    };
    let setup = PostgresContextSetup::construct(config);
    let ctx = PostgresContext::new(setup).build().await?;
    let client = ctx.postgres().client().await?;

    sqlx::query("CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT NOT NULL)")
        .execute(&client)
        .await?;

    sqlx::query("INSERT INTO test_table (name) VALUES ($1)")
        .bind("test_name")
        .execute(&client)
        .await?;

    let name: String = sqlx::query_scalar("SELECT name FROM test_table WHERE id = 1")
        .fetch_one(&client)
        .await?;

    assert_eq!(name, "test_name", "Should retrieve the inserted name");

    ctx.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_postgres_context_health_check() -> eyre::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    let config = PostgresContextConfig {
        postgres: Postgres::default(),
    };
    let setup = PostgresContextSetup::construct(config);
    let ctx = PostgresContext::new(setup).build().await?;

    ctx.postgres().healthy().await?;

    ctx.stop().await?;

    Ok(())
}
