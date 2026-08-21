use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn request_password_reset_returns_a_202_if_called_with_non_existing_email() {
    let app = spawn_app().await;

    let body = r#"{
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_reset_password_request(body.into()).await;

    assert_eq!(response.status().as_u16(), 202);
}

#[tokio::test]
async fn request_password_reset_returns_a_202_if_called_with_existing_email() {
    let app = spawn_app().await;

    let body_user = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    let body_reset = r#"{
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // create user
    app.post_signup(body_user.into()).await;

    // request password reset
    let response = app.post_reset_password_request(body_reset.into()).await;

    assert_eq!(response.status().as_u16(), 202);
}

#[tokio::test]
async fn request_password_reset_sends_a_password_reset_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;

    let body_user = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    let body_reset = r#"{
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    // create user
    app.post_signup(body_user.into()).await;

    // request password reset
    app.post_reset_password_request(body_reset.into()).await;

    // mock asserts if there were 2 emails sent
}

#[tokio::test]
async fn request_password_reset_sends_a_password_reset_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;

    let body_user = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    let body_reset = r#"{
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // create a user
    app.post_signup(body_user.into()).await;

    // request password reset
    app.post_reset_password_request(body_reset.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    app.get_activation_link(&email_request);
}

#[tokio::test]
async fn request_password_reset_fails_if_there_is_a_fatal_database_error() {
    let app = spawn_app().await;

    let body = r#"{
        "email": "acid@house.net"
    }"#;

    // destroy the  database
    sqlx::query!("ALTER TABLE users DROP COLUMN email;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app.post_reset_password_request(body.into()).await;

    assert_eq!(response.status().as_u16(), 500);
}
