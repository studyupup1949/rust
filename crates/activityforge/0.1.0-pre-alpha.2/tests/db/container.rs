use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicI16, Ordering};

use activityforge::{
    Error, Result,
    db::{Db, DbConfig},
};

pub static DB_STARTED: AtomicI16 = AtomicI16::new(0);
pub static DB_READY: AtomicBool = AtomicBool::new(false);
pub static DB_RETRIES: AtomicI16 = AtomicI16::new(1024);

pub const DB_CONTAINER: &str = "activityforge_test_db";

/// Starts a test database container.
#[rustfmt::skip]
pub fn start_db(config: &DbConfig) -> Result<()> {
    if cfg!(feature = "ci") || DB_STARTED.fetch_add(1, Ordering::AcqRel) > 0 {
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
                "postgres:latest",
            ])
            .output()?;

        Ok(())
    }
}

/// Waits in a busy loop while the database starts up.
pub async fn wait_for_db(config: &DbConfig) -> Result<()> {
    if DB_READY.load(Ordering::Acquire) {
        Ok(())
    } else {
        while let Err(err) = Db::test_connect(config.clone()).await {
            log::debug!("waiting for a successful db connection: {err}");

            if DB_RETRIES.fetch_sub(1, Ordering::AcqRel) >= 1 {
                tokio::time::sleep(core::time::Duration::from_millis(128)).await;
            } else {
                return Err(Error::io("max DB retries exceeded"));
            }
        }

        DB_READY.store(true, Ordering::Release);

        Ok(())
    }
}

/// Stops the test database container.
pub fn stop_db() -> Result<()> {
    // decrease the "DB_STARTED" container for each test-suite that called `start_db`
    // on the last call "DB_STARTED" previous value should be `1`, actually stop the container
    if cfg!(feature = "ci") || DB_STARTED.fetch_sub(1, Ordering::AcqRel) > 1 {
        Ok(())
    } else {
        Command::new("podman")
            .args(["container", "stop", DB_CONTAINER])
            .output()
            .map(|_| ())
            .map_err(Error::from)
    }
}
