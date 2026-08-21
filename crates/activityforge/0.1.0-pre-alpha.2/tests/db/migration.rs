use std::sync::atomic::{AtomicBool, Ordering};

use activityforge::{Error, Result, db::Db};

static MIGRATION_COMPLETE: AtomicBool = AtomicBool::new(false);

/// Tests the base and integration test DB migrations.
pub async fn test_migration(db: &Db) -> Result<()> {
    if migration_complete() {
        Ok(())
    } else {
        let pool = db.pool()?;

        let path = std::path::PathBuf::from("migrations/v1");

        let migrate = sqlx::migrate::Migrator::new(path).await?;
        migrate.run(pool).await.map_err(Error::from).map(|_| {
            set_migration_complete(true);
        })
    }
}

/// Gets whether the database migration is complete.
pub fn migration_complete() -> bool {
    MIGRATION_COMPLETE.load(Ordering::Acquire)
}

/// Sets whether the database migration is complete.
pub fn set_migration_complete(val: bool) {
    MIGRATION_COMPLETE.store(val, Ordering::Release)
}
