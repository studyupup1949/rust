use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn the_link_returned_by_activate_resend_returns_a_200_if_called() {
    let app = spawn_app().await;

    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    let resend_body = r#"{
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_signup(body.into()).await;

    app.post_activate_resend(resend_body.into()).await;

    // get activation link from resend request
    let email_request = &app.email_server.received_requests().await.unwrap()[1];
    let activation_link = app.get_activation_link(&email_request);

    let response = reqwest::get(activation_link).await.unwrap();

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_resent_activation_link_activates_a_user() {
    let app = spawn_app().await;
    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    let resend_body = r#"{
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_signup(body.into()).await;

    app.post_activate_resend(resend_body.into()).await;

    // get activation link from resend request
    let email_request = &app.email_server.received_requests().await.unwrap()[1];
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
async fn activate_resend_returns_202_if_email_is_not_used() {
    let app = spawn_app().await;
    let resend_body = r#"{
        "email": "acid@house.net"
    }"#;

    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_activate_resend(resend_body.into()).await;

    assert_eq!(response.status().as_u16(), 202);
}
