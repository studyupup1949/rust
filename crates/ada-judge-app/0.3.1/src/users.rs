use crate::{
    AJAlertType, AJAppState, AJMainWindow, AJUserState, alert::show_alert, app_state::AppState,
    load_all,
};
use ada_judge_public_models::{
    DeletionRequest,
    users::{AdminLevel, PrivateUserData, PublicUserData},
};
use keyring::Entry;
use reqwest::StatusCode;
use slint::{ComponentHandle, ToSharedString, Weak};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn load_account_data(state: Arc<Mutex<AppState>>, weak: Weak<AJMainWindow>) {
    if let Err(e) = load_account_data_impl(state.clone(), weak.clone()).await {
        show_alert(weak.clone(), e, AJAlertType::Error);
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
            show_alert(weak.clone(), e, AJAlertType::Error);
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

pub fn load_user_data(id: i64, state: Arc<Mutex<AppState>>, weak: Weak<AJMainWindow>) {
    tokio::spawn(async move {
        if let Err(e) = load_user_data_impl(id, state.clone(), weak.clone()).await {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn load_user_data_impl(
    id: i64,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let current_user = state.lock().await.user.clone().unwrap();
    if current_user.admin_level != AdminLevel::Owner {
        let user = client
            .get(format!("{base_url}/users/{id}"))
            .bearer_auth(token)
            .send()
            .await?
            .json::<PublicUserData>()
            .await?;
        slint::invoke_from_event_loop(move || {
            if let Some(ui) = weak.upgrade() {
                ui.global::<AJAppState>()
                    .set_opened_user_public_profile(user.into());
                ui.global::<AJAppState>().set_page(7);
            }
        })
        .unwrap();
    } else {
        let user = client
            .get(format!("{base_url}/users/{id}/private"))
            .bearer_auth(token)
            .send()
            .await?
            .json::<PrivateUserData>()
            .await?;
        slint::invoke_from_event_loop(move || {
            if let Some(ui) = weak.upgrade() {
                ui.global::<AJAppState>()
                    .set_opened_user_private_profile(user.into());
                ui.global::<AJAppState>().set_page(8);
            }
        })
        .unwrap();
    }

    Ok(())
}

pub fn delete_my_account(
    login: String,
    password: String,
    password_confirmation: String,
    deletion_confirmation: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    tokio::spawn(async move {
        if let Err(e) = delete_my_account_impl(
            login,
            password,
            password_confirmation,
            deletion_confirmation,
            state.clone(),
            weak.clone(),
        )
        .await
        {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn delete_my_account_impl(
    login: String,
    password: String,
    password_confirmation: String,
    deletion_confirmation: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let request = DeletionRequest {
        login,
        password,
        password_confirmation,
        deletion_confirmation,
    };
    let res = client
        .delete(format!("{base_url}/users/me/delete_account"))
        .bearer_auth(token.clone())
        .json(&request)
        .send()
        .await?;

    logout(state.clone(), weak.clone());

    match res.status() {
        StatusCode::OK => {
            show_alert(
                weak.clone(),
                "успешно".to_shared_string(),
                AJAlertType::Success,
            );
        }
        _ => {
            show_alert(
                weak.clone(),
                "ошибка".to_shared_string(),
                AJAlertType::Error,
            );
        }
    }

    Ok(())
}

pub fn delete_user_account(
    id: i32,
    login: String,
    password: String,
    password_confirmation: String,
    deletion_confirmation: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    tokio::spawn(async move {
        if let Err(e) = delete_user_account_impl(
            id,
            login,
            password,
            password_confirmation,
            deletion_confirmation,
            state.clone(),
            weak.clone(),
        )
        .await
        {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn delete_user_account_impl(
    id: i32,
    login: String,
    password: String,
    password_confirmation: String,
    deletion_confirmation: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let request = DeletionRequest {
        login,
        password,
        password_confirmation,
        deletion_confirmation,
    };
    let res = client
        .delete(format!("{base_url}/users/{id}/delete_account"))
        .bearer_auth(token.clone())
        .json(&request)
        .send()
        .await?;

    {
        let weak = weak.clone();

        slint::invoke_from_event_loop(move || {
            if let Some(ui) = weak.upgrade() {
                ui.global::<AJAppState>().set_page(5);
            }
        })
        .unwrap();
    }

    match res.status() {
        StatusCode::OK => {
            show_alert(
                weak.clone(),
                "успешно".to_shared_string(),
                AJAlertType::Success,
            );
        }
        _ => {
            show_alert(
                weak.clone(),
                "ошибка".to_shared_string(),
                AJAlertType::Error,
            );
        }
    }
    Ok(())
}

pub fn change_user_admin_level(
    id: i32,
    admin_level: AdminLevel,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    tokio::spawn(async move {
        if let Err(e) =
            change_user_admin_level_impl(id, admin_level, state.clone(), weak.clone()).await
        {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn change_user_admin_level_impl(
    id: i32,
    admin_level: AdminLevel,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let res = client
        .patch(format!("{base_url}/users/{id}/change_admin_level"))
        .bearer_auth(token.clone())
        .json(&admin_level)
        .send()
        .await?;

    {
        let weak = weak.clone();

        slint::invoke_from_event_loop(move || {
            if let Some(ui) = weak.upgrade() {
                let all_contests = ui.global::<AJAppState>().get_all_contests();
                let opened_user_id = ui
                    .global::<AJAppState>()
                    .get_opened_user_private_profile()
                    .id as i64;

                tokio::spawn(async move {
                    load_all(all_contests, state.clone(), weak.clone()).await;
                    load_user_data(opened_user_id, state.clone(), weak.clone());
                });
            }
        })
        .unwrap();
    }

    match res.status() {
        StatusCode::OK => {
            show_alert(
                weak.clone(),
                "успешно".to_shared_string(),
                AJAlertType::Success,
            );
        }
        _ => {
            show_alert(
                weak.clone(),
                "ошибка".to_shared_string(),
                AJAlertType::Error,
            );
        }
    }
    Ok(())
}
