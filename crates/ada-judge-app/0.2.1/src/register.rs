use crate::{AJMainWindow, alert::show_alert, app_state::AppState};
use ada_judge_public_models::users::RegisterRequest;
use anyhow::anyhow;
use reqwest::StatusCode;
use slint::{SharedString, Weak};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn register(
    login: SharedString,
    password: SharedString,
    password_confirmation: SharedString,
    master_password: SharedString,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    tokio::spawn(async move {
        if let Err(e) = register_impl(
            login,
            password,
            password_confirmation,
            master_password,
            state.clone(),
            weak.clone(),
        )
        .await
        {
            show_alert(weak.clone(), e, crate::AJAlertType::Error);
        }
    });
}

pub async fn register_impl(
    login: SharedString,
    password: SharedString,
    password_confirmation: SharedString,
    master_password: SharedString,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let request = RegisterRequest {
        login: login.to_string(),
        password: password.to_string(),
        password_confirmation: password_confirmation.to_string(),
        master_password: master_password.to_string(),
    };
    let client = reqwest::Client::new();
    let base_url = state.lock().await.base_url.clone();
    let res = client
        .post(format!("{base_url}/register"))
        .json(&request)
        .send()
        .await?;
    match res.status() {
        StatusCode::OK => {
            show_alert(weak.clone(), "успешно", crate::AJAlertType::Success);
        }
        _ => {
            return Err(anyhow!("ошибка регистрации"));
        }
    }

    Ok(())
}
