use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn the_link_returned_by_activate_returns_a_200_if_called() {
    let app = spawn_app().await;

    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_signup(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let activation_link = app.get_activation_link(&email_request);

    let response = reqwest::get(activation_link).await.unwrap();

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn activations_without_token_are_rejected_with_a_400() {
    let app = spawn_app().await;

    let response = reqwest::get(&format!("{}/api/v1/auth/signup/activate", app.address))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn clicking_on_the_activation_link_activates_a_user() {
    let app = spawn_app().await;

    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_signup(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let activation_link = app.get_activation_link(&email_request);

    reqwest::get(activation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT username, email, status FROM users WHERE username='db303'",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved user.");

    assert_eq!(saved.email, "acid@house.net");
    assert_eq!(saved.status, "active");
}

#[tokio::test]
async fn clicking_on_the_used_activation_link_returns_401() {
    let app = spawn_app().await;
    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_signup(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let activation_link = app.get_activation_link(&email_request);

    reqwest::get(activation_link.clone()).await.unwrap();

    let response = reqwest::get(activation_link).await.unwrap();
    assert_eq!(response.status().as_u16(), 401);
}
