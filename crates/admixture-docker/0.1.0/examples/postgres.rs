//! Example: PostgreSQL Integration with sqlx
//!
//! This example demonstrates how to use admixture with a PostgreSQL Docker container
//! and sqlx for database operations.
//!
//! Run with: cargo run -p admixture-docker --features sqlx-postgres --example postgres

#[cfg(feature = "sqlx-postgres")]
use admixture::service::{ServiceRunning, ServiceSetup};
#[cfg(feature = "sqlx-postgres")]
use admixture_docker::SqlxPostgresServiceSetup;
#[cfg(feature = "sqlx-postgres")]
use testcontainers_modules::postgres::Postgres;

#[cfg(feature = "sqlx-postgres")]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    println!("🐘 PostgreSQL Integration Example\n");

    let setup = SqlxPostgresServiceSetup::construct(Postgres::default());

    println!("Starting PostgreSQL container...");
    let mut running = setup.start().await?;
    println!("✅ PostgreSQL started\n");

    let client = running.client().await?;

    println!("Running query: SELECT 1");
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&client)
        .await?;

    println!("✅ Query result: {}\n", result);

    println!("Stopping PostgreSQL...");
    running.stop().await?;
    println!("✅ PostgreSQL stopped");

    Ok(())
}

#[cfg(not(feature = "sqlx-postgres"))]
fn main() {
    eprintln!("This example requires the 'sqlx-postgres' feature.");
    eprintln!("Run with: cargo run -p admixture-docker --features sqlx-postgres --example postgres");
    std::process::exit(1);
}
