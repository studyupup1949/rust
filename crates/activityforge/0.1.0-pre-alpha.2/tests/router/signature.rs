use activityforge::Result;
use activityforge::app::App;
use activityforge::db::DbConfig;
use activitystreams_vocabulary::{Iri, Name};

use http::{Method, StatusCode};

use crate::db::{connect, container, migration};

use super::{mock_server, test_server_port};

#[tokio::test]
async fn test_signature() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .try_init()
        .ok();

    let db_host = std::env::var("POSTGRES_HOST").unwrap_or("127.0.0.1".to_string());
    let username = std::env::var("POSTGRES_USER").unwrap_or("activityforge_test".to_string());
    let password = std::env::var("POSTGRES_PASSWORD").unwrap_or("activityforge_test".to_string());
    let db_name = std::env::var("POSTGRES_DB_NAME").unwrap_or("activityforge_test".to_string());
    let port: u16 = 5432;

    let config = DbConfig::new()
        .with_username(username)
        .with_password(password)
        .with_host(db_host)
        .with_port(port)
        .with_db_name(db_name);

    container::start_db(&config)?;
    container::wait_for_db(&config).await?;

    let db = connect::test_connection(&config).await?;
    migration::test_migration(&db).await?;

    let app_port = test_server_port();
    let app_host = format!("127.0.0.1:{app_port}");

    let app = App::create(
        config,
        Iri::try_from(format!("http://{app_host}")).map(|i| i.into())?,
        Name::try_from("router_signature_test").map(|n| n.into())?,
    )
    .await
    .map_err(|err| {
        log::error!("error creating app server: {err}");
    })
    .unwrap();

    mock_server().await?;

    let server = app.test_router().await.unwrap();
    let listener = tokio::net::TcpListener::bind(&app_host).await.unwrap();

    tokio::spawn(async move {
        axum::serve(listener, server).await.ok();
    });

    let app_uri = app.uri();

    // test signed request validates properly
    let res = app
        .state()
        .signed_request::<()>(
            Method::GET,
            &format!("{app_uri}/signature").try_into().unwrap(),
            None,
        )
        .await
        .map_err(|err| {
            log::error!("error with self-signed request: {err}");
            err
        })?;
    assert_eq!(res.status(), StatusCode::OK);
    log::info!("tests: router: successfully verified self-signed request");

    container::stop_db()
}
