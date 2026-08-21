use crate::{AJContestState, AJLeaderboardRow, AJMainWindow, app_state::AppState};
use ada_judge_public_models::{contests::LeaderboardRow, users::PublicUserData};
use futures::future::try_join_all;
use reqwest::StatusCode;
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn load_contest_leaderboard(
    id: i64,
    state: Arc<Mutex<AppState>>,
    weak: Weak<AJMainWindow>,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let token = state.lock().await.token.clone();
    let base_url = state.lock().await.base_url.clone();
    let res = client
        .get(format!("{base_url}/contests/{id}/leaderboard"))
        .bearer_auth(token.clone())
        .send()
        .await?;
    if res.status() != StatusCode::OK {
        return Ok(());
    }
    let rows = res.json::<Vec<LeaderboardRow>>().await?;
    let rows: Vec<(String, LeaderboardRow)> = try_join_all(rows.iter().map(|row| {
        let client = client.clone();
        let token = token.clone();
        let base_url = base_url.clone();

        async move {
            let user = client
                .get(format!("{base_url}/users/{}", row.user_id))
                .bearer_auth(token.clone())
                .send()
                .await?
                .json::<PublicUserData>()
                .await?;
            Ok::<_, anyhow::Error>((user.login, row.clone()))
        }
    }))
    .await?;
    slint::invoke_from_event_loop(move || {
        let rows: Vec<AJLeaderboardRow> = rows
            .iter()
            .map(|(username, row)| (username.clone(), row).into())
            .collect();

        if let Some(ui) = weak.upgrade() {
            ui.global::<AJContestState>()
                .set_leaderboard_rows(ModelRc::new(VecModel::from(rows)));
        }
    })
    .unwrap();

    Ok(())
}
