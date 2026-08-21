use activityforge::{Error, Result, db::Db};

/// Tests the base and integration test DB migrations.
pub async fn test_migration(db: &Db) -> Result<()> {
    let pool = db.pool()?;

    let path = std::path::PathBuf::from("migrations/v1");

    let migrate = sqlx::migrate::Migrator::new(path).await?;
    migrate.run(pool).await.map_err(Error::from).map(|_| ())
}
