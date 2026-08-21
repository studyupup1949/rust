use activityforge::app::App;
use activityforge::crypto::{
    AlgorithmName, DigestAlgorithm, HttpContentDigest, HttpMessageComponentId,
    HttpMessageSignature, HttpPrivateKey, HttpSignatureParams,
};
use activityforge::db::DbConfig;
use activityforge::{Error, Result};
use activitystreams_vocabulary::{Iri, Name};

use axum::extract::Request;

use bytes::Bytes;
use http::{Method, StatusCode};

use crate::db::{connect, container, migration};

use super::{ED25519_KEY_ID, ED25519_PRIVKEY_BYTES, mock_server, test_server_port};

#[tokio::test]
async fn router_middleware_tests() -> Result<()> {
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
        Name::try_from("middleware_router_tests").map(|n| n.into())?,
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

    middleware_request(&app, ED25519_KEY_ID, Method::GET, "fetch-actor", None).await?;
    middleware_request(
        &app,
        ED25519_KEY_ID,
        Method::GET,
        "fetch-actor",
        Some("multikey"),
    )
    .await?;
    middleware_request(
        &app,
        ED25519_KEY_ID,
        Method::GET,
        "fetch-actor",
        Some("pem"),
    )
    .await?;

    container::stop_db()
}

/// Creates a signed request to test HTTP Message Signature middleware.
async fn middleware_request(
    app: &App,
    key_id: &str,
    method: Method,
    path: &str,
    encoding: Option<&str>,
) -> Result<()> {
    let privkey =
        HttpPrivateKey::from_bytes(key_id, AlgorithmName::Ed25519, &ED25519_PRIVKEY_BYTES)?;

    let sig_params = HttpSignatureParams::try_new(&[
        HttpMessageComponentId::try_from("date")?,
        HttpMessageComponentId::try_from("content-digest")?,
    ])
    .map(|mut s| {
        s.set_keyid(key_id);
        s
    })?;

    let app_uri = app.uri();

    let body = r#"{"test": "httpsig-middleware"}"#;

    let date = "Mon, 19 Apr 2021 02:07:55 GMT";

    let req: Request<Bytes> = Request::builder()
        .method(method)
        .uri(format!("{app_uri}/{path}").as_str())
        .header("date", date)
        .body(Bytes::from(body))?;

    let mut req = req.set_content_digest(DigestAlgorithm::Sha512).await?;
    let sig_name = "sig-test-middleware";

    req.set_message_signature(&sig_params, &privkey, Some(sig_name))?;

    let client = reqwest::Client::new();

    let req = client
        .get(req.uri().to_string())
        .headers(req.headers().clone());

    let req = if let Some(encoding) = encoding {
        req.header("key-encoding", encoding)
    } else {
        req
    };

    let res = req
        .body(body)
        .send()
        .await
        .map_err(|err| Error::http(format!("error making test connection: {err}")))?;

    assert_eq!(res.status(), StatusCode::OK);

    Ok(())
}
