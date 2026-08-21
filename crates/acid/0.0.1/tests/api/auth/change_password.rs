use {
    crate::helpers::spawn_app,
    wiremock::{
        matchers::{method, path},
        Mock, ResponseTemplate,
    },
};

#[tokio::test]
async fn change_password_changes_password_if_token_is_correct() {
    let app = spawn_app().await;

    let body_new_user = r#"{
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
    app.post_signup(body_new_user.into()).await;

    // activate user
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let activation_link = app.get_activation_link(&email_request);

    reqwest::get(activation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // request password reset
    app.post_reset_password_request(body_reset.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[1];
    let reset_token = app.get_reset_token(&email_request);

    // change password
    app.post_change_password(
        serde_json::json!({
            "reset_token": reset_token,
            "password": "House!808",
            "password_again": "House!808"
        })
        .to_string(),
    )
    .await;

    // try to log in with a new password
    let response = app
        .post_login(
            serde_json::json!({
                "username": "db303",
                "password": "House!808"
            })
            .to_string(),
        )
        .await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn change_password_returns_200_if_the_latest_token_is_used() {
    let app = spawn_app().await;

    let body_new_user = r#"{
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
    app.post_signup(body_new_user.into()).await;

    // activate user
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let activation_link = app.get_activation_link(&email_request);

    reqwest::get(activation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // request password reset twice
    app.post_reset_password_request(body_reset.into()).await;
    app.post_reset_password_request(body_reset.into()).await;

    // get reset token from the latest request
    let email_request = &app.email_server.received_requests().await.unwrap()[2];
    let reset_token = app.get_reset_token(&email_request);

    // change password
    app.post_change_password(
        serde_json::json!({
            "reset_token": reset_token,
            "password": "House!808",
            "password_again": "House!808"
        })
        .to_string(),
    )
    .await;

    // try to log in with a new password
    let response = app
        .post_login(
            serde_json::json!({
                "username": "db303",
                "password": "House!808"
            })
            .to_string(),
        )
        .await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn change_password_returns_401_if_another_token_was_requested() {
    let app = spawn_app().await;

    let body_new_user = r#"{
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
    app.post_signup(body_new_user.into()).await;

    // activate user
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let activation_link = app.get_activation_link(&email_request);

    reqwest::get(activation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // request password reset two times
    app.post_reset_password_request(body_reset.into()).await;
    app.post_reset_password_request(body_reset.into()).await;

    // get reset token from the latest request
    let email_request = &app.email_server.received_requests().await.unwrap()[1];
    let reset_token = app.get_reset_token(&email_request);

    // change password
    let response = app
        .post_change_password(
            serde_json::json!({
                "reset_token": reset_token,
                "password": "House!808",
                "password_again": "House!808"
            })
            .to_string(),
        )
        .await;

    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn change_password_returns_400_if_new_password_is_invalid() {
    let app = spawn_app().await;

    let body_new_user = r#"{
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
    app.post_signup(body_new_user.into()).await;

    // activate user
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let activation_link = app.get_activation_link(&email_request);

    reqwest::get(activation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // request password reset two times
    app.post_reset_password_request(body_reset.into()).await;

    // get reset token
    let email_request = &app.email_server.received_requests().await.unwrap()[1];
    let reset_token = app.get_reset_token(&email_request);

    // change password
    let response = app
        .post_change_password(
            serde_json::json!({
                "reset_token": reset_token,
                "password": "H",
                "password_again": "H"
            })
            .to_string(),
        )
        .await;

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn change_password_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;

    let no_reset_token = r#"{
        "password": "House!909",
        "password_again": "House!909"
    }"#;

    let no_password = r#"{
        "reset_token": "12346",
        "new_password_again": "House!909"
    }"#;

    let no_password_repeat = r#"{
        "reset_token": "12346",
        "password": "House!909",
    }"#;

    let test_cases = vec![
        (no_reset_token, "missing the reset token"),
        (no_password, "missing the password"),
        (no_password_repeat, "missing the password repeat"),
        ("", "missing everything"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_change_password(invalid_body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn change_password_returns_a_401_if_passwords_do_not_match() {
    let app = spawn_app().await;
    let body = r#"{
        "reset_token": "12346",
        "password": "House!909",
        "password_again": "House!303"
    }"#;

    let response = app.post_change_password(body.into()).await;

    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn change_password_returns_a_401_if_reset_token_is_invalid() {
    let app = spawn_app().await;
    let body = r#"{
        "reset_token": "12346",
        "password": "House!909",
        "password_again": "House!909"
    }"#;

    let response = app.post_change_password(body.into()).await;

    assert_eq!(401, response.status().as_u16());
}
