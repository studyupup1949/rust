//! The `adminx` command-line tool.
//!
//! Built only with the `cli` feature (`cargo install adminx --features cli`).
//! It bundles both storage backends, so one binary manages admin users for
//! Postgres/MySQL/SQLite (SeaORM) and MongoDB. The backend is chosen at runtime
//! from the environment:
//!
//!   Postgres/MySQL/SQLite:  DATABASE_URL=postgres://user:pass@host/db
//!   MongoDB:                MONGO_URL=mongodb://host:port  [MONGO_DB=adminx]
//!
//! Example:
//!   DATABASE_URL=postgres://u:p@localhost/app \
//!     adminx create-admin -e admin@example.com -p changeme

use adminx::prelude::*;
use clap::{Parser, Subcommand};

/// Admin table/collection managed by the CLI.
const USERS_TABLE: &str = "adminx_users";

#[derive(Parser)]
#[command(name = "adminx", version, about = "adminx admin-panel CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create an admin user (idempotent — skips if the email already exists).
    CreateAdmin {
        /// Email address (or set EMAIL).
        #[arg(short, long, env = "EMAIL")]
        email: String,
        /// Password (or set PASSWORD).
        #[arg(short, long, env = "PASSWORD")]
        password: String,
        /// Role (or set ROLE).
        #[arg(short, long, env = "ROLE", default_value = "admin")]
        role: String,
    },

    /// Seed the database from a file (or stdin). SQL statements for SeaORM
    /// backends, or JSON command documents for Mongo — one statement per line;
    /// blank lines and `--`/`#` comments are ignored.
    Seed {
        /// Path to the seed file. Reads stdin when omitted.
        #[arg(short, long)]
        file: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    match Cli::parse().command {
        Command::CreateAdmin {
            email,
            password,
            role,
        } => create_admin_cmd(&email, &password, &role).await,
        Command::Seed { file } => seed_cmd(file).await,
    }
}

async fn seed_cmd(file: Option<String>) -> anyhow::Result<()> {
    use std::io::Read;

    // Read the seed source (file or stdin), then one statement per non-blank,
    // non-comment line.
    let raw = match &file {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("reading {path}: {e}"))?,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };
    let statements: Vec<String> = raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("--") && !l.starts_with('#'))
        .map(str::to_owned)
        .collect();
    if statements.is_empty() {
        anyhow::bail!("no statements to seed (empty input)");
    }
    let refs: Vec<&str> = statements.iter().map(String::as_str).collect();

    if let Ok(url) = std::env::var("DATABASE_URL") {
        let n = adminx::seaorm::seed(&url, &refs)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        println!("✓ seeded {} statement(s) — {n} row(s) affected", refs.len());
    } else if let Some(uri) = env_any(&["MONGO_URL", "MONGODB_URL"]) {
        let db = env_any(&["MONGO_DB", "ADMINX_DB_NAME"]).unwrap_or_else(|| "adminx".into());
        let n = adminx::mongo::seed(&uri, &db, &refs)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;
        println!("✓ seeded {} statement(s) — {n} document(s) affected", refs.len());
    } else {
        anyhow::bail!(
            "no storage configured: set DATABASE_URL (SQL) or MONGO_URL (MongoDB)"
        );
    }
    Ok(())
}

async fn create_admin_cmd(email: &str, password: &str, role: &str) -> anyhow::Result<()> {
    init_storage().await?;

    // create_admin() reads admin_table from here; jwt_secret is unused for seeding.
    configure_auth(AuthConfig {
        jwt_secret: "cli-seed".into(),
        token_ttl_secs: 86_400,
        admin_table: USERS_TABLE.into(),
        secure_cookie: false,
    });

    let existing = storage()
        .find_one_by(USERS_TABLE, "email", email)
        .await
        .map_err(CoreError::from)?;
    if existing.is_some() {
        println!("user {email} already exists — nothing to do");
        return Ok(());
    }

    create_admin(email, password, role).await?;
    println!("✓ created {role} {email}");
    Ok(())
}

/// Choose a storage backend from the environment and register it. For SeaORM
/// backends the admin table is created if missing (with the MFA columns).
async fn init_storage() -> anyhow::Result<()> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        let store = adminx::seaorm::connect(&url).await?;
        store.execute_sql(users_ddl(&url)).await?;
        set_storage(Box::new(store));
        Ok(())
    } else if let Some(uri) = env_any(&["MONGO_URL", "MONGODB_URL"]) {
        // Mongo is schemaless — the collection and fields appear on first insert.
        let db = env_any(&["MONGO_DB", "ADMINX_DB_NAME"]).unwrap_or_else(|| "adminx".into());
        adminx::mongo::init(&uri, &db).await?;
        Ok(())
    } else {
        anyhow::bail!(
            "no storage configured: set DATABASE_URL (Postgres/MySQL/SQLite) \
             or MONGO_URL (MongoDB)"
        )
    }
}

fn env_any(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| std::env::var(k).ok())
}

/// `CREATE TABLE IF NOT EXISTS` for the admin users, dialect-adjusted by URL scheme.
fn users_ddl(url: &str) -> &'static str {
    if url.starts_with("sqlite") {
        "CREATE TABLE IF NOT EXISTS adminx_users (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            email TEXT NOT NULL UNIQUE, \
            encrypted_password TEXT NOT NULL, \
            role TEXT NOT NULL DEFAULT 'admin', \
            mfa_enabled BOOLEAN NOT NULL DEFAULT 0, \
            mfa_secret TEXT, \
            mfa_backup_codes TEXT)"
    } else if url.starts_with("mysql") {
        "CREATE TABLE IF NOT EXISTS adminx_users (\
            id INT AUTO_INCREMENT PRIMARY KEY, \
            email VARCHAR(255) NOT NULL UNIQUE, \
            encrypted_password TEXT NOT NULL, \
            role VARCHAR(64) NOT NULL DEFAULT 'admin', \
            mfa_enabled BOOLEAN NOT NULL DEFAULT false, \
            mfa_secret TEXT, \
            mfa_backup_codes TEXT)"
    } else {
        // Postgres (and the default).
        "CREATE TABLE IF NOT EXISTS adminx_users (\
            id SERIAL PRIMARY KEY, \
            email TEXT NOT NULL UNIQUE, \
            encrypted_password TEXT NOT NULL, \
            role TEXT NOT NULL DEFAULT 'admin', \
            mfa_enabled BOOLEAN NOT NULL DEFAULT false, \
            mfa_secret TEXT, \
            mfa_backup_codes TEXT)"
    }
}
