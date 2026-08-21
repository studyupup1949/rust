use crate::{
    AJAlertType, AJContestState, AJMainWindow, AJProblem, AJProblemsState, alert::show_alert,
    app_state::AppState,
};
use ada_judge_public_models::{DeletionRequest, problems::PublicProblemConfig};
use futures::future::try_join_all;
use reqwest::{
    StatusCode,
    multipart::{Form, Part},
};
use slint::{ComponentHandle, ModelRc, SharedString, ToSharedString, VecModel, Weak};
use std::{fs, sync::Arc};
use tokio::sync::Mutex;

pub async fn load_contest_problems(
    id: i64,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let ids = client
        .get(format!("{base_url}/contests/{id}/problems"))
        .bearer_auth(token.clone())
        .send()
        .await?
        .json::<Vec<i64>>()
        .await?;
    let problems: Vec<PublicProblemConfig> = try_join_all(ids.iter().map(|problem_id| {
        let client = client.clone();
        let token = token.clone();
        let base_url = base_url.clone();

        async move {
            client
                .get(format!("{base_url}/contests/{id}/problems/{problem_id}"))
                .bearer_auth(token.clone())
                .send()
                .await?
                .json::<PublicProblemConfig>()
                .await
        }
    }))
    .await?;
    state.lock().await.problems = problems.clone();
    let user = state.lock().await.user.clone().unwrap();
    slint::invoke_from_event_loop(move || {
        let problems: Vec<AJProblem> = problems.iter().map(|x| (x, &user).into()).collect();
        let problems_names: Vec<SharedString> = problems
            .iter()
            .map(|x| format!("#{} {}", x.problem_index + 1, x.name).to_shared_string())
            .collect();

        if let Some(ui) = weak.upgrade() {
            ui.global::<AJContestState>()
                .set_problems(ModelRc::new(VecModel::from(problems)));
            ui.global::<AJContestState>()
                .set_problems_names(ModelRc::new(VecModel::from(problems_names)));
        }
    })
    .unwrap();

    Ok(())
}

pub fn retest_problem_submissions(id: i64, state: Arc<Mutex<AppState>>, weak: Weak<AJMainWindow>) {
    tokio::spawn(async move {
        if let Err(e) = retest_problem_submissions_impl(id, state.clone(), weak.clone()).await {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn retest_problem_submissions_impl(
    id: i64,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let res = client
        .post(format!("{base_url}/problems/{id}/retest-submissions"))
        .bearer_auth(token.clone())
        .send()
        .await?;

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

pub async fn load_problems(
    all_problems: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let ids = if all_problems {
        client
            .get(format!("{base_url}/problems"))
            .bearer_auth(token.clone())
            .send()
            .await?
            .json::<Vec<i64>>()
            .await?
    } else {
        client
            .get(format!("{base_url}/problems/my"))
            .bearer_auth(token.clone())
            .send()
            .await?
            .json::<Vec<i64>>()
            .await?
    };
    let problems: Vec<PublicProblemConfig> = try_join_all(ids.iter().map(|problem_id| {
        let client = client.clone();
        let token = token.clone();
        let base_url = base_url.clone();

        async move {
            client
                .get(format!("{base_url}/problems/{problem_id}",))
                .bearer_auth(token.clone())
                .send()
                .await?
                .json::<PublicProblemConfig>()
                .await
        }
    }))
    .await?;
    let user = state.lock().await.user.clone().unwrap();
    slint::invoke_from_event_loop(move || {
        let problems: Vec<AJProblem> = problems.iter().map(|x| (x, &user).into()).collect();

        if let Some(ui) = weak.upgrade() {
            ui.global::<AJProblemsState>()
                .set_problems(ModelRc::new(VecModel::from(problems)));
        }
    })
    .unwrap();

    Ok(())
}

pub fn create_problem(state: Arc<Mutex<AppState>>, weak: Weak<AJMainWindow>) {
    tokio::spawn(async move {
        if let Err(e) = create_problem_impl(state.clone(), weak.clone()).await {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn create_problem_impl(
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let file = fs::read(state.lock().await.problem_archive.clone())?;
    let form = Form::new().part("problem_archive", Part::bytes(file));
    let res = client
        .post(format!("{base_url}/problems/new"))
        .bearer_auth(token.clone())
        .multipart(form)
        .send()
        .await?;

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

pub fn delete_problem(
    id: i32,
    login: String,
    password: String,
    password_confirmation: String,
    deletion_confirmation: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    tokio::spawn(async move {
        if let Err(e) = delete_problem_impl(
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

pub async fn delete_problem_impl(
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
        .delete(format!("{base_url}/problems/{id}/delete"))
        .bearer_auth(token.clone())
        .json(&request)
        .send()
        .await?;

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
