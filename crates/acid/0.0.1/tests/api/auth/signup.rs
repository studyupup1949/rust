use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn signup_returns_a_201_for_valid_json_data() {
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

    let response = app.post_signup(body.into()).await;

    assert_eq!(201, response.status().as_u16());
}

#[tokio::test]
async fn signup_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;

    let no_username = r#"{
        "password": "house909",
        "emai": "acid@house.net"
    }"#;

    let no_password = r#"{
        "username": "db303",
        "emai": "acid@house.net"
    }"#;

    let no_email = r#"{
        "username": "db303",
        "password": "house909",
    }"#;

    let test_cases = vec![
        (no_email, "missing the email"),
        (no_username, "missing the username"),
        (no_password, "missing the password"),
        ("", "missing everything"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = app.post_signup(invalid_body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn signup_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let empty_username = r#"{
        "username": "",
        "password": "House!909",
        "emai": "acid@house.net"
    }"#;
    let empty_password = r#"{
        "username": "db303",
        "password": "",
        "emai": "acid@house.net"
    }"#;
    let empty_email = r#"{
        "username": "db303",
        "password": "House!909",
        "emai": ""
    }"#;
    let wrong_username = r#"{
        "username": "very loooooooooooooooooooooooooooooooooooooong",
        "password": "House!909",
        "emai": "acid@house.net"
    }"#;
    let wrong_password = r#"{
        "username": "db303",
        "password": "1",
        "emai": "acid@house.net"
    }"#;
    let wrong_email = r#"{
        "username": "db303",
        "password": "House!909",
        "emai": "this is not an email"
    }"#;
    let test_cases = vec![
        (empty_username, "empty username"),
        (empty_email, "empty email"),
        (empty_password, "empty password"),
        (wrong_username, "wrong username"),
        (wrong_password, "wrong password"),
        (wrong_email, "wrong email"),
    ];
    for (body, description) in test_cases {
        let response = app.post_signup(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn signup_persists_the_new_user() {
    // Arrange
    let app = spawn_app().await;
    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    app.post_signup(body.into()).await;

    let saved =
        sqlx::query!("SELECT username, email, password_hash FROM users WHERE username='db303'",)
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch saved user.");

    assert_eq!(saved.username, "db303");
    assert_eq!(saved.email, "acid@house.net");
}

#[tokio::test]
async fn signup_returns_a_409_if_email_is_taken() {
    let app = spawn_app().await;
    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;
    app.post_signup(body.into()).await;

    let body = r#"{
        "username": "db808",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    let response = app.post_signup(body.into()).await;

    assert_eq!(409, response.status().as_u16());
}

#[tokio::test]
async fn signup_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;
    Mock::given(path("/send"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_signup(body.into()).await;

    // mock asserts on drop
}

#[tokio::test]
async fn signup_sends_a_confirmation_email_with_a_link() {
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

    app.get_activation_link(&email_request);
}

#[tokio::test]
async fn signup_fails_if_there_is_a_fatal_database_error() {
    // Arrange
    let app = spawn_app().await;

    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    // destroy the  database
    sqlx::query!("ALTER TABLE users DROP COLUMN password_hash;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app.post_signup(body.into()).await;

    // assert
    assert_eq!(response.status().as_u16(), 500);
}
