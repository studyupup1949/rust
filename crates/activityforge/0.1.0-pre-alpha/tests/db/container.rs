use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use activityforge::{Error, Result, db::DbConfig};

pub static DB_STARTED: AtomicBool = AtomicBool::new(false);
pub const DB_CONTAINER: &str = "activityforge_test_db";

/// Starts a test database container.
#[rustfmt::skip]
pub fn start_db(config: &DbConfig) -> Result<()> {
    if cfg!(feature = "ci") || DB_STARTED.load(Ordering::Acquire) {
        Ok(())
    } else {
        // start the database container
        Command::new("podman")
            .args([
                "run",
                "-d",
                "--rm",
                "--name", DB_CONTAINER,
                "-e", format!("POSTGRES_USER={}", config.username()).as_str(),
                "-e", format!("POSTGRES_PASSWORD={}", config.password()).as_str(),
                "-e", format!("POSTGRES_DB_NAME={}", config.db_name()).as_str(),
                "-p", "127.0.0.1:5432:5432",
                "postgres:15-bookworm",
            ])
            .output()?;

        // wait until the database is ready
        Command::new("bash")
            .args([
                "-c",
                format!("until podman exec {DB_CONTAINER} pg_isready; do sleep 1; done").as_str(),
            ])
            .output()?;

        DB_STARTED.store(true, Ordering::Release);

        Ok(())
    }
}

/// Stops the test database container.
pub fn stop_db() -> Result<()> {
    if cfg!(feature = "ci") || !DB_STARTED.load(Ordering::Acquire) {
        Ok(())
    } else {
        Command::new("podman")
            .args(["container", "stop", DB_CONTAINER])
            .output()
            .map(|_| {
                DB_STARTED.store(false, Ordering::Release);
            })
            .map_err(Error::from)
    }
}
