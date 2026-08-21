use crate::{
    AJAlertType, AJAppState, AJContestState, AJMainWindow, AJSubmission, alert::show_alert,
    app_state::AppState, load_all,
};
use ada_judge_public_models::{
    testing::{Language, Submission, SubmissonRequest},
    users::PublicUserData,
};
use futures::{StreamExt, future::try_join_all};
use reqwest::{
    StatusCode,
    multipart::{Form, Part},
};
use rfd::AsyncFileDialog;
use slint::{ComponentHandle, ModelRc, ToSharedString, VecModel, Weak};
use std::{fs, sync::Arc};
use tokio::{fs::File, io::AsyncWriteExt, sync::Mutex};

pub async fn load_problem_submissions(
    problem_id: i64,
    all_submissions: bool,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let ids = if all_submissions {
        client
            .get(format!(
                "{base_url}/submissions/filter/problem/{problem_id}"
            ))
            .bearer_auth(token.clone())
            .send()
            .await?
            .json::<Vec<i64>>()
            .await?
    } else {
        client
            .get(format!(
                "{base_url}/submissions/my/filter/problem/{problem_id}"
            ))
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
    let submissions: Vec<(String, Submission)> = try_join_all(submissions.iter().map(|sbm| {
        let client = client.clone();
        let token = token.clone();
        let base_url = base_url.clone();

        async move {
            let user = client
                .get(format!("{base_url}/users/{}", sbm.user_id))
                .bearer_auth(token.clone())
                .send()
                .await?
                .json::<PublicUserData>()
                .await?;
            Ok::<_, anyhow::Error>((user.login, sbm.clone()))
        }
    }))
    .await?;
    slint::invoke_from_event_loop(move || {
        let submissions: Vec<AJSubmission> = submissions
            .iter()
            .map(|(username, sbm)| (username.clone(), sbm).into())
            .collect();

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

pub fn download_submission(index: i32, state: Arc<Mutex<AppState>>, weak: Weak<AJMainWindow>) {
    tokio::spawn(async move {
        if let Err(e) = download_submission_impl(index, state.clone(), weak.clone()).await {
            show_alert(weak.clone(), e, AJAlertType::Error);
        }
    });
}

#[must_use]
pub const fn get_language_file_extension(language: &Language) -> &'static str {
    match language {
        Language::Clang => "c",
        Language::Clangpp => "cpp",
        Language::Go => "go",
        Language::Rust => "rs",
        Language::Python => "py",
        Language::Unknown => "!!",
    }
}

pub async fn download_submission_impl(
    index: i32,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();

    let submission = (&state.lock().await.submissions[index as usize]).clone();
    if let Some(path) = AsyncFileDialog::new()
        .set_file_name(format!(
            "solution.{}",
            get_language_file_extension(&submission.language)
        ))
        .save_file()
        .await
    {
        let res = client
            .get(format!("{base_url}/submissions/{}/download", submission.id))
            .bearer_auth(token.clone())
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let mut stream = res.bytes_stream();
                let mut file = File::create(path.path()).await?;

                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    file.write_all(&chunk).await?;
                }

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
    }

    Ok(())
}
