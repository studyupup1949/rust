use activityforge::db::DbConfig;

pub fn test_config() -> DbConfig {
    let db_host = std::env::var("POSTGRES_HOST").unwrap_or("127.0.0.1".to_string());
    let username = std::env::var("POSTGRES_USER").unwrap_or("activityforge_test".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or("activityforge_test".to_string());
    let db_name = std::env::var("POSTGRES_DB_NAME").unwrap_or("activityforge_test".to_string());
    let port = std::env::var("POSTGRES_PORT")
        .unwrap_or("5432".to_string())
        .parse::<u16>()
        .unwrap_or(5432u16);

    DbConfig::new()
        .with_username(username)
        .with_password(password)
        .with_host(db_host)
        .with_port(port)
        .with_db_name(db_name)
}
