use crate::{
    AJAlertType, AJContestState, AJMainWindow, AJSubmission, alert::show_alert, app_state::AppState,
};
use ada_judge_public_models::testing::{Language, Submission, SubmissonRequest};
use futures::future::try_join_all;
use reqwest::{
    StatusCode,
    multipart::{Form, Part},
};
use slint::{ComponentHandle, ModelRc, ToSharedString, VecModel, Weak};
use std::{fs, sync::Arc};
use tokio::sync::Mutex;

pub async fn load_contest_submissions(
    id: i64,
    all_submissions: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let ids = if all_submissions {
        client
            .get(format!("{base_url}/submissions/filter/contest/{id}"))
            .bearer_auth(token.clone())
            .send()
            .await?
            .json::<Vec<i64>>()
            .await?
    } else {
        client
            .get(format!("{base_url}/submissions/my/filter/contest/{id}"))
            .bearer_auth(token.clone())
            .send()
            .await?
            .json::<Vec<i64>>()
            .await?
    };
    let submissions: Vec<Submission> = try_join_all(ids.iter().map(|submission_id| {
        let client = client.clone();
        let token = token.clone();
        let base_url = base_url.clone();

        async move {
            client
                .get(format!("{base_url}/submissions/{submission_id}"))
                .bearer_auth(token.clone())
                .send()
                .await?
                .json::<Submission>()
                .await
        }
    }))
    .await?;
    state.lock().await.submissions = submissions.clone();
    slint::invoke_from_event_loop(move || {
        let submissions: Vec<AJSubmission> = submissions.iter().map(|sbm| sbm.into()).collect();

        if let Some(ui) = weak.upgrade() {
            ui.global::<AJContestState>()
                .set_submissions(ModelRc::new(VecModel::from(submissions)));
        }
    })
    .unwrap();

    Ok(())
}

pub fn submit(
    contest_id: i64,
    problem_id: i64,
    language: Language,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) {
    tokio::spawn(async move {
        if let Err(e) = submit_impl(
            contest_id,
            problem_id,
            language,
            state.clone(),
            weak.clone(),
        )
        .await
        {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

pub async fn submit_impl(
    contest_id: i64,
    problem_id: i64,
    language: Language,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let request = SubmissonRequest {
        problem_id,
        language,
    };
    let json = serde_json::to_string(&request)?;
    let file = fs::read(state.lock().await.solution_file.clone())?;
    let form = Form::new()
        .part("submission_data", Part::text(json))
        .part("submission_file", Part::bytes(file));
    let res = client
        .post(format!("{base_url}/contests/{contest_id}/submit"))
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
