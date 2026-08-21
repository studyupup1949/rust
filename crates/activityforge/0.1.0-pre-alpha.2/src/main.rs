use std::net::SocketAddr;

use axum_server::tls_rustls::RustlsConfig;
use tokio::net::TcpListener;

use activityforge::app::App;
use activityforge::db::{DbConfig, Iri, Name};
use activityforge::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .try_init()
        .ok();

    let db_host = std::env::var("POSTGRES_HOST").unwrap_or("127.0.0.1".to_string());
    let username = std::env::var("POSTGRES_USER").unwrap_or("activityforge_test".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or("activityforge_test".to_string());
    let db_name = std::env::var("POSTGRES_DB_NAME").unwrap_or("activityforge_test".to_string());
    let db_port: u16 = std::env::var("POSTGRES_PORT")
        .unwrap_or("5432".to_string())
        .parse()
        .map_err(|err| Error::http(format!("error parsing DB port: {err}")))?;

    let host = std::env::var("ACTIVITYFORGE_HOST").unwrap_or("127.0.0.1".to_string());
    let port: u16 = std::env::var("ACTIVITYFORGE_PORT")
        .unwrap_or("3000".to_string())
        .parse()
        .map_err(|err| Error::http(format!("error parsing app port: {err}")))?;

    let dev_mode: bool = std::env::var("ACTIVITYFORGE_DEV")
        .unwrap_or("true".to_string())
        .parse()
        .map_err(|err| Error::http(format!("error parsing app dev mode: {err}")))?;

    let scheme = if dev_mode { "http" } else { "https" };

    let app_id = std::env::var("ACTIVITYFORGE_APP_ID")
        .map_err(|err| Error::http(format!("{err}")))
        .or(Ok(format!("{scheme}://{host}:{port}")))
        .and_then(Iri::try_from)?;

    let app_name = std::env::var("ACTIVITYFORGE_APP_NAME")
        .map_err(|err| Error::http(format!("{err}")))
        .or(Ok("activityforge".to_string()))
        .and_then(Name::try_from)?;

    let config = DbConfig::new()
        .with_username(username)
        .with_password(password)
        .with_host(db_host)
        .with_port(db_port)
        .with_db_name(db_name);

    let app = App::create(config, app_id, app_name).await?;

    let server = app.router().await?;

    let cert_file = std::env::var("ACTIVITYFORGE_CERT_PEM").ok();
    let key_file = std::env::var("ACTIVITYFORGE_KEY_PEM").ok();

    if let (Some(cert), Some(key)) = (cert_file, key_file) {
        let addr: SocketAddr = format!("{host}:{port}")
            .parse()
            .map_err(|err| Error::http(format!("error parsing listening address: {err}")))?;

        let config = RustlsConfig::from_pem_file(cert, key).await?;

        axum_server::bind_rustls(addr, config)
            .serve(server.into_make_service())
            .await
            .map_err(|err| Error::http(format!("error serving app: {err}")))
    } else {
        let listener = TcpListener::bind(format!("{host}:{port}")).await?;

        axum::serve(listener, server)
            .await
            .map_err(|err| Error::http(format!("error serving app: {err}")))
    }
}
