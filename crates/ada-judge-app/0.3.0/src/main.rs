//! Native desktop app for `ada-judge`

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]

use crate::{
    alert::show_alert,
    app_state::AppState,
    contests::{create_contest, delete_contest, update_contest},
    leaderboard::load_contest_leaderboard,
    login::login,
    problems::{create_problem, delete_problem, load_contest_problems, load_problems},
    register::register,
    submissions::{download_submission, load_problem_submissions},
    users::{
        change_user_admin_level, delete_my_account, delete_user_account, load_account_data,
        load_user_data, logout,
    },
};
use ada_judge_public_models::users::AdminLevel;
use contests::load_contests;
use keyring::Entry;
use problems::retest_problem_submissions;
use rfd::AsyncFileDialog;
use slint::{Model, ToSharedString, Weak};
use std::{env, error::Error, sync::Arc};
use submissions::submit;
use tokio::sync::Mutex;

mod alert;
mod app_state;
mod contests;
mod converts;
mod leaderboard;
mod login;
mod problems;
mod register;
mod submissions;
mod users;

slint::include_modules!();

async fn load_all(all_contests: bool, state: Arc<Mutex<AppState>>, weak: Weak<AJMainWindow>) {
    load_account_data(state.clone(), weak.clone()).await;
    load_contests(all_contests, state.clone(), weak.clone()).await;
    let user = state.lock().await.user.clone();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = weak.upgrade() {
            let contest_index = ui.global::<AJContestState>().get_index();
            let in_contest = ui.global::<AJAppState>().get_in_contest();
            let problem_index = ui.global::<AJContestState>().get_current_problem_index() as usize;
            let all_submissions = ui.global::<AJContestState>().get_all_submissions();
            if in_contest {
                let contest_id = ui
                    .global::<AJAppState>()
                    .get_contests()
                    .iter()
                    .collect::<Vec<AJContest>>()[contest_index as usize]
                    .id as i64;
                {
                    let state = state.clone();
                    let weak = weak.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            load_contest_problems(contest_id, state.clone(), weak.clone()).await
                        {
                            show_alert(weak.clone(), e, AJAlertType::Error);
                        }
                        let problem_id =
                            state.lock().await.problems.clone()[problem_index].id as i64;
                        if let Err(e) = load_problem_submissions(
                            problem_id,
                            all_submissions,
                            state.clone(),
                            weak.clone(),
                        )
                        .await
                        {
                            show_alert(weak.clone(), e, AJAlertType::Error);
                        }
                        if let Err(e) =
                            load_contest_leaderboard(contest_id, state.clone(), weak.clone()).await
                        {
                            show_alert(weak.clone(), e, AJAlertType::Error);
                        }
                    });
                }
            }
            if let Some(user) = user {
                ui.global::<AJAppState>().set_can_create_contests(
                    user.admin_level >= ada_judge_public_models::users::AdminLevel::Admin,
                );
                let all_problems = ui.global::<AJProblemsState>().get_all_problems();
                let weak = weak.clone();
                tokio::spawn(async move {
                    if user.admin_level >= AdminLevel::Admin {
                        if let Err(e) =
                            load_problems(all_problems, state.clone(), weak.clone()).await
                        {
                            show_alert(weak.clone(), e, AJAlertType::Error);
                        }
                    }
                });
            }
        }
        show_alert(weak.clone(), "успешно", AJAlertType::Success);
    })
    .unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ui = AJMainWindow::new()?;
    let state = Arc::new(Mutex::new(AppState::default()));
    state.lock().await.base_url =
        env::var("BASE_URL").unwrap_or("https://contest.oneprog.org".into());
    let weak = ui.as_weak();
    match Entry::new("com.oneprog.ada-judge", "jwt") {
        Ok(entry) => match entry.get_password() {
            Ok(token) => {
                state.lock().await.token = token.trim_matches('"').to_string();
                let all_contests = ui.global::<AJAppState>().get_all_contests();
                load_all(all_contests, state.clone(), weak.clone()).await;
                if state.lock().await.user.is_some() {
                    ui.global::<AJAppState>().set_page(3);
                }
            }
            Err(_) => {}
        },
        Err(_) => {}
    }
    ui.global::<AJAppState>()
        .set_version(env!("CARGO_PKG_VERSION").to_shared_string());
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJWelcomeCallbacks>()
            .on_login(move |login_str, password_str| {
                login(login_str, password_str, weak.clone(), state.clone());
            });
    }
    {
        let weak = weak.clone();
        let state = state.clone();
        ui.global::<AJWelcomeCallbacks>().on_register(
            move |login_str, password_str, password_confirmation_str| {
                register(
                    login_str,
                    password_str,
                    password_confirmation_str,
                    state.clone(),
                    weak.clone(),
                );
            },
        );
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJHomeCallbacks>()
            .on_enter_contest(move |index| {
                let state_async = state.clone();
                let weak_async = weak.clone();

                if let Some(ui) = weak.upgrade() {
                    let all_contests = ui.global::<AJAppState>().get_all_contests();

                    ui.global::<AJContestState>().set_index(index);
                    tokio::spawn(async move {
                        load_all(all_contests, state_async.clone(), weak_async.clone()).await;
                        {
                            let weak = weak_async.clone();
                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = weak.upgrade() {
                                    ui.global::<AJAppState>().set_page(5);
                                }
                            })
                            .unwrap();
                        }
                    });
                }
            });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJContestCallbacks>()
            .on_pick_solution_file(move || {
                let state = state.clone();
                let weak = weak.clone();

                tokio::spawn(async move {
                    match AsyncFileDialog::new().set_directory("/").pick_file().await {
                        Some(handle) => {
                            state.lock().await.solution_file = handle.path().to_path_buf();
                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = weak.upgrade() {
                                    ui.global::<AJContestState>().set_solution_picked(true);
                                    ui.global::<AJContestState>().set_solution_name(
                                        handle
                                            .path()
                                            .file_name()
                                            .unwrap()
                                            .to_string_lossy()
                                            .to_shared_string(),
                                    );
                                }
                            })
                            .unwrap();
                        }
                        None => (),
                    }
                });
            });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJAppCallbacks>().on_reload(move || {
            let state = state.clone();
            let weak = weak.clone();

            if let Some(ui) = weak.upgrade() {
                let all_contests = ui.global::<AJAppState>().get_all_contests();

                tokio::spawn(async move {
                    load_all(all_contests, state.clone(), weak.clone()).await;
                });
            }
        });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJContestCallbacks>()
            .on_submit(move |language, contest_id, problem_id| {
                let language = language.into();

                submit(
                    contest_id as i64,
                    problem_id as i64,
                    language,
                    state.clone(),
                    weak.clone(),
                )
            });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJContestCallbacks>()
            .on_retest_problem(move |problem_id| {
                retest_problem_submissions(problem_id as i64, state.clone(), weak.clone())
            });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJHomeCallbacks>().on_create_contest(
            move |name,
                  starts_at,
                  ends_at,
                  statements_url,
                  editorial_url,
                  hidden,
                  upsolving_opened,
                  hide_solutions| {
                create_contest(
                    name.to_string(),
                    starts_at,
                    ends_at,
                    statements_url.to_string(),
                    editorial_url.to_string(),
                    hidden,
                    upsolving_opened,
                    hide_solutions,
                    state.clone(),
                    weak.clone(),
                )
            },
        );
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJHomeCallbacks>().on_update_contest(
            move |id,
                  name,
                  starts_at,
                  ends_at,
                  statements_url,
                  editorial_url,
                  hidden,
                  upsolving_opened,
                  hide_solutions| {
                update_contest(
                    id,
                    name.to_string(),
                    starts_at,
                    ends_at,
                    statements_url.to_string(),
                    editorial_url.to_string(),
                    hidden,
                    upsolving_opened,
                    hide_solutions,
                    state.clone(),
                    weak.clone(),
                )
            },
        );
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJHomeCallbacks>().on_delete_contest(
            move |id, login, password, password_confirmation, deletion_confirmation| {
                delete_contest(
                    id,
                    login.to_string(),
                    password.to_string(),
                    password_confirmation.to_string(),
                    deletion_confirmation,
                    state.clone(),
                    weak.clone(),
                );
            },
        );
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJProblemsCallbacks>()
            .on_pick_problem_archive(move || {
                let state = state.clone();
                let weak = weak.clone();

                tokio::spawn(async move {
                    match AsyncFileDialog::new()
                        .set_directory("/")
                        .add_filter("архив", &["zip"])
                        .pick_file()
                        .await
                    {
                        Some(handle) => {
                            state.lock().await.problem_archive = handle.path().to_path_buf();
                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = weak.upgrade() {
                                    ui.global::<AJProblemsState>().set_problem_picked(true);
                                    ui.global::<AJProblemsState>().set_problem_archive_name(
                                        handle
                                            .path()
                                            .file_name()
                                            .unwrap()
                                            .to_string_lossy()
                                            .to_shared_string(),
                                    );
                                }
                            })
                            .unwrap();
                        }
                        _ => (),
                    }
                });
            });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJProblemsCallbacks>()
            .on_create_problem(move || {
                create_problem(state.clone(), weak.clone());
            });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJHomeCallbacks>().on_logout(move || {
            logout(state.clone(), weak.clone());
        });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJContestCallbacks>()
            .on_open_profile(move |id| {
                load_user_data(id as i64, state.clone(), weak.clone());
            });
    }
    {
        let weak = weak.clone();
        ui.global::<AJAppCallbacks>().on_open_link(move |url| {
            if let Err(e) = open::that(url) {
                show_alert(weak.clone(), e, AJAlertType::Error);
            }
        });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJProblemsCallbacks>().on_delete_problem(
            move |id, login, password, password_confirmation, deletion_confirmation| {
                delete_problem(
                    id,
                    login.to_string(),
                    password.to_string(),
                    password_confirmation.to_string(),
                    deletion_confirmation,
                    state.clone(),
                    weak.clone(),
                );
            },
        );
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJHomeCallbacks>().on_delete_account(
            move |login, password, password_confirmation, deletion_confirmation| {
                delete_my_account(
                    login.to_string(),
                    password.to_string(),
                    password_confirmation.to_string(),
                    deletion_confirmation,
                    state.clone(),
                    weak.clone(),
                );
            },
        );
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJContestCallbacks>().on_delete_user_account(
            move |id, login, password, password_confirmation, deletion_confirmation| {
                delete_user_account(
                    id,
                    login.to_string(),
                    password.to_string(),
                    password_confirmation.to_string(),
                    deletion_confirmation,
                    state.clone(),
                    weak.clone(),
                );
            },
        );
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJContestCallbacks>()
            .on_change_user_admin_level(move |id, admin_level| {
                change_user_admin_level(id, admin_level.into(), state.clone(), weak.clone());
            });
    }
    {
        let state = state.clone();
        let weak = weak.clone();
        ui.global::<AJContestCallbacks>()
            .on_download_submission(move |index| {
                download_submission(index, state.clone(), weak.clone());
            });
    }
    ui.run()?;
    Ok(())
}
