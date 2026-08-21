use ada_judge_public_models::{
    contests::PublicContestConfig, problems::PublicProblemConfig, testing::Submission,
    users::PrivateUserData,
};
use std::path::PathBuf;

#[derive(Default)]
pub struct AppState {
    pub token: String,
    pub user: Option<PrivateUserData>,
    pub contests: Vec<PublicContestConfig>,
    pub problems: Vec<PublicProblemConfig>,
    pub submissions: Vec<Submission>,
    pub solution_file: PathBuf,
    pub problem_archive: PathBuf,
    pub base_url: String,
}
