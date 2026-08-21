use crate::{
    AJAlertType, AJAppState, AJContest, AJContestState, AJDateTime, AJMainWindow,
    alert::show_alert, app_state::AppState, load_all,
};
use ada_judge_public_models::{
    DeletionRequest,
    contests::{ContestRequest, PublicContestConfig},
};
use chrono::{DateTime, Local};
use futures::future::try_join_all;
use reqwest::StatusCode;
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel, Weak};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn load_contests(
    all_contests: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    if let Err(e) = load_contests_impl(all_contests, state.clone(), weak.clone()).await {
        show_alert(weak.clone(), e, AJAlertType::Error);
    }
}

pub async fn load_contests_impl(
    all_contests: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let ids = if all_contests {
        client
            .get(format!("{base_url}/contests"))
            .bearer_auth(token.clone())
            .send()
            .await?
            .json::<Vec<i64>>()
            .await?
    } else {
        client
            .get(format!("{base_url}/contests/my"))
            .bearer_auth(token.clone())
            .send()
            .await?
            .json::<Vec<i64>>()
            .await?
    };
    let contests: Vec<PublicContestConfig> = try_join_all(ids.iter().map(|id| {
        let client = client.clone();
        let token = token.clone();
        let base_url = base_url.clone();

        async move {
            client
                .get(format!("{base_url}/contests/{id}"))
                .bearer_auth(token.clone())
                .send()
                .await?
                .json::<PublicContestConfig>()
                .await
        }
    }))
    .await?;
    state.lock().await.contests = contests.clone();
    let user = state.lock().await.user.clone().unwrap();
    let contests: Vec<AJContest> = contests.iter().map(|x| (x, &user).into()).collect();
    let contests_names: Vec<SharedString> = contests
        .iter()
        .map(|x| format!("#{} {}", x.id, x.name).to_shared_string())
        .collect();

    slint::invoke_from_event_loop(move || {
        if let Some(ui) = weak.upgrade() {
            ui.global::<AJAppState>()
                .set_contests(ModelRc::new(VecModel::from(contests)));
            ui.global::<AJAppState>()
                .set_contests_names(ModelRc::new(VecModel::from(contests_names)));
        }
    })
    .unwrap();

    Ok(())
}

pub fn create_contest(
    name: String,
    starts_at: AJDateTime,
    ends_at: AJDateTime,
    statements_url: String,
    editorial_url: String,
    hidden: bool,
    upsolving_opened: bool,
    hide_solutions: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    tokio::spawn(async move {
        if let Err(e) = create_contest_impl(
            name,
            starts_at,
            ends_at,
            statements_url,
            editorial_url,
            hidden,
            upsolving_opened,
            hide_solutions,
            state.clone(),
            weak.clone(),
        )
        .await
        {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn create_contest_impl(
    name: String,
    starts_at: AJDateTime,
    ends_at: AJDateTime,
    statements_url: String,
    editorial_url: String,
    hidden: bool,
    upsolving_opened: bool,
    hide_solutions: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let starts_at: DateTime<Local> = starts_at.try_into()?;
    let ends_at: DateTime<Local> = ends_at.try_into()?;

    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let request = ContestRequest {
        name,
        starts_at: starts_at.to_utc(),
        ends_at: ends_at.to_utc(),
        statements_url,
        editorial_url,
        hidden,
        upsolving_opened,
        hide_solutions,
    };
    let res = client
        .post(format!("{base_url}/contests/new"))
        .bearer_auth(token.clone())
        .json(&request)
        .send()
        .await?;

    {
        let weak = weak.clone();

        slint::invoke_from_event_loop(move || {
            if let Some(ui) = weak.upgrade() {
                let all_contests = ui.global::<AJAppState>().get_all_contests();

                tokio::spawn(async move {
                    load_all(all_contests, state.clone(), weak.clone()).await;
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

pub fn update_contest(
    id: i32,
    name: String,
    starts_at: AJDateTime,
    ends_at: AJDateTime,
    statements_url: String,
    editorial_url: String,
    hidden: bool,
    upsolving_opened: bool,
    hide_solutions: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    tokio::spawn(async move {
        if let Err(e) = update_contest_impl(
            id,
            name,
            starts_at,
            ends_at,
            statements_url,
            editorial_url,
            hidden,
            upsolving_opened,
            hide_solutions,
            state.clone(),
            weak.clone(),
        )
        .await
        {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn update_contest_impl(
    id: i32,
    name: String,
    starts_at: AJDateTime,
    ends_at: AJDateTime,
    statements_url: String,
    editorial_url: String,
    hidden: bool,
    upsolving_opened: bool,
    hide_solutions: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let starts_at: DateTime<Local> = starts_at.try_into()?;
    let ends_at: DateTime<Local> = ends_at.try_into()?;

    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let request = ContestRequest {
        name,
        starts_at: starts_at.to_utc(),
        ends_at: ends_at.to_utc(),
        statements_url,
        editorial_url,
        hidden,
        upsolving_opened,
        hide_solutions,
    };
    let res = client
        .patch(format!("{base_url}/contests/{id}/update"))
        .bearer_auth(token.clone())
        .json(&request)
        .send()
        .await?;

    {
        let weak = weak.clone();

        slint::invoke_from_event_loop(move || {
            if let Some(ui) = weak.upgrade() {
                let all_contests = ui.global::<AJAppState>().get_all_contests();

                tokio::spawn(async move {
                    load_all(all_contests, state.clone(), weak.clone()).await;
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

pub fn delete_contest(
    id: i32,
    login: String,
    password: String,
    password_confirmation: String,
    deletion_confirmation: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    tokio::spawn(async move {
        if let Err(e) = delete_contest_impl(
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

pub async fn delete_contest_impl(
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
        .delete(format!("{base_url}/contests/{id}/delete"))
        .bearer_auth(token.clone())
        .json(&request)
        .send()
        .await?;

    {
        let weak = weak.clone();

        slint::invoke_from_event_loop(move || {
            if let Some(ui) = weak.upgrade() {
                if ui.global::<AJAppState>().get_in_contest()
                    && id
                        == ui
                            .global::<AJAppState>()
                            .get_contests()
                            .iter()
                            .collect::<Vec<AJContest>>()
                            [ui.global::<AJContestState>().get_index() as usize]
                            .id
                {
                    ui.global::<AJAppState>().set_in_contest(false);
                    ui.global::<AJAppState>().set_page(3);
                    ui.global::<AJContestState>().invoke_reset();
                }
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
