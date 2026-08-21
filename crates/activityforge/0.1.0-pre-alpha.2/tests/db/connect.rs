use activityforge::{
    Result,
    db::{Db, DbConfig},
};

/// Basic test to ensure a basic database connection.
///
/// Requires calling `start_db` first to start the test DB container.
pub async fn test_connection(config: &DbConfig) -> Result<Db> {
    Db::connect(config.clone()).await
}
