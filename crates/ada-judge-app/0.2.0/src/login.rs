use crate::{
    AJAlertType, AJAppState, AJMainWindow, alert::show_alert, app_state::AppState, load_all,
};
use ada_judge_public_models::users::LoginRequest;
use anyhow::anyhow;
use keyring::Entry;
use reqwest::StatusCode;
use slint::{ComponentHandle, SharedString, Weak};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn login(
    login: SharedString,
    password: SharedString,
    weak: Weak<AJMainWindow>,
    state: Arc<Mutex<AppState>>,
) {
    tokio::spawn(async move {
        if let Err(e) = login_impl(login, password, weak.clone(), state.clone()).await {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn login_impl(
    login: SharedString,
    password: SharedString,
    weak: Weak<AJMainWindow>,
    state: Arc<Mutex<AppState>>,
) -> Result<(), anyhow::Error> {
    let request = LoginRequest {
        login: login.to_string(),
        password: password.to_string(),
    };
    let client = reqwest::Client::new();
    let base_url = state.lock().await.base_url.clone();
    let res = client
        .post(format!("{base_url}/login"))
        .json(&request)
        .send()
        .await?;
    match res.status() {
        StatusCode::OK => {
            let token = res.text().await?;
            let entry = Entry::new("com.oneprog.ada-judge", "jwt")?;
            entry.set_password(&token)?;
            state.lock().await.token = token.trim_matches('"').to_string();
            load_all(state.clone(), weak.clone()).await;
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = weak.upgrade() {
                    ui.global::<AJAppState>().set_page(3);
                }
            })
            .unwrap();
        }
        _ => {
            return Err(anyhow!("неверный логин или пароль"));
        }
    }

    Ok(())
}
