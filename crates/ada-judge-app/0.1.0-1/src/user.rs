use crate::{AJAppState, AJMainWindow, AJUserState, alert::show_alert, app_state::AppState};
use ada_judge_public_models::users::PrivateUserData;
use keyring::Entry;
use slint::{ComponentHandle, Weak};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn load_account_data(state: Arc<Mutex<AppState>>, weak: Weak<AJMainWindow>) {
    if let Err(e) = load_account_data_impl(state.clone(), weak.clone()).await {
        show_alert(weak.clone(), e, crate::AJAlertType::Error);
    }
}

pub async fn load_account_data_impl(
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let user = client
        .get(format!("{base_url}/users/me"))
        .bearer_auth(token)
        .send()
        .await?
        .json::<PrivateUserData>()
        .await?;
    state.lock().await.user = Some(user.clone());
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = weak.upgrade() {
            ui.global::<AJUserState>().set_user(user.into());
        }
    })
    .unwrap();

    Ok(())
}

pub fn logout(state: Arc<Mutex<AppState>>, weak: Weak<AJMainWindow>) {
    tokio::spawn(async move {
        if let Err(e) = logout_impl(state.clone(), weak.clone()).await {
            show_alert(weak.clone(), e, crate::AJAlertType::Error);
        }
    });
}

pub async fn logout_impl(
    _state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let entry = Entry::new("com.oneprog.ada-judge", "jwt")?;
    entry.delete_credential()?;

    slint::invoke_from_event_loop(move || {
        if let Some(ui) = weak.upgrade() {
            ui.global::<AJAppState>().set_page(0);
        }
    })
    .unwrap();

    Ok(())
}
