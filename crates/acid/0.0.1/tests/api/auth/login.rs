use crate::helpers::spawn_app;

#[tokio::test]
async fn login_returns_401_if_credentials_are_invalid() {
    let app = spawn_app().await;

    let body = r#"{
         "username": "&app.test_user.username",
         "password": "&app.test_user.password"
    }"#;

    let response = app.post_login(body.into()).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn login_returns_200_if_credentials_are_valid() {
    let app = spawn_app().await;

    let body = serde_json::json!({
         "username": &app.test_user.username,
         "password": &app.test_user.password
    });

    let response = app.post_login(serde_json::to_string(&body).unwrap()).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn login_returns_403_if_account_is_not_activated() {
    let app = spawn_app().await;

    let body = r#"{
        "username": "db303",
        "password": "House!909",
        "email": "acid@house.net"
    }"#;

    app.post_signup(body.into()).await;

    let response = app.post_login(body.into()).await;

    assert_eq!(response.status().as_u16(), 403);
}
