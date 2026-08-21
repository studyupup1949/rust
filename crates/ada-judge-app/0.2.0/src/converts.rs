use crate::{
    AJAccountData, AJAdminLevel, AJContest, AJContestStatus, AJDateTime, AJLanguage,
    AJLeaderboardRow, AJProblem, AJSubgroup, AJSubgroupResult, AJSubgroupType, AJSubgroupVerdict,
    AJSubmission, AJTotalVerdict,
};
use ada_judge_public_models::{
    contests::{LeaderboardRow, PublicContestConfig},
    problems::{PublicProblemConfig, Subgroup, SubgroupType},
    testing::{Language, SubgroupResult, Submission},
    users::{AdminLevel, PrivateUserData},
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use anyhow::anyhow;
use chrono::{DateTime, Datelike, Timelike, Utc};
use slint::{ModelRc, SharedString, ToSharedString, VecModel};
use std::cmp::Ordering;

impl From<DateTime<Utc>> for AJDateTime {
    fn from(date_time: DateTime<Utc>) -> Self {
        Self {
            day: date_time.day() as i32,
            hour: date_time.hour() as i32,
            minute: date_time.minute() as i32,
            month: date_time.month() as i32,
            second: date_time.second() as i32,
            year: date_time.year() as i32,
        }
    }
}

impl TryFrom<AJDateTime> for DateTime<Utc> {
    type Error = anyhow::Error;

    fn try_from(date_time: AJDateTime) -> Result<Self, anyhow::Error> {
        Utc::now()
            .with_year(date_time.year)
            .ok_or_else(|| anyhow!("некорретный год"))?
            .with_month(date_time.month as u32)
            .ok_or_else(|| anyhow!("некорретный месяц"))?
            .with_day(date_time.day as u32)
            .ok_or_else(|| anyhow!("некорретный день"))?
            .with_hour(date_time.hour as u32)
            .ok_or_else(|| anyhow!("некорретный час"))?
            .with_minute(date_time.minute as u32)
            .ok_or_else(|| anyhow!("некорретная минута"))?
            .with_second(date_time.second as u32)
            .ok_or_else(|| anyhow!("некорретная секунда"))
    }
}

impl From<(&PublicContestConfig, &PrivateUserData)> for AJContest {
    fn from((contest, user): (&PublicContestConfig, &PrivateUserData)) -> Self {
        let now = Utc::now();

        Self {
            can_manage: contest.owner_id.is_some_and(|owner_id| owner_id == user.id)
                || user.admin_level == AdminLevel::Owner,
            ends_at: contest.ends_at.into(),
            id: contest.id as i32,
            name: contest.name.to_shared_string(),
            owner_id: contest.owner_id.unwrap_or(-1) as i32,
            starts_at: contest.starts_at.into(),
            statements_url: contest.statements_url.to_shared_string(),
            status: match now.cmp(&contest.starts_at) {
                Ordering::Less => AJContestStatus::BeforeStart,
                _ => match now.cmp(&contest.ends_at) {
                    Ordering::Less | Ordering::Equal => AJContestStatus::Ongoing,
                    _ => AJContestStatus::Finished,
                },
            },
        }
    }
}

impl From<&Subgroup> for AJSubgroup {
    fn from(subgroup: &Subgroup) -> Self {
        Self {
            depends_on: subgroup
                .depends_on
                .iter()
                .map(|x| x.to_shared_string())
                .collect::<Vec<SharedString>>()
                .join(", ")
                .to_shared_string(),
            score: subgroup.score,
            tests: subgroup
                .tests
                .iter()
                .map(|x| x.to_shared_string())
                .collect::<Vec<SharedString>>()
                .join(", ")
                .to_shared_string(),
            r#type: match subgroup.r#type {
                SubgroupType::Main => AJSubgroupType::Main,
                _ => AJSubgroupType::Sample,
            },
        }
    }
}

impl From<(&PublicProblemConfig, &PrivateUserData)> for AJProblem {
    fn from((problem, user): (&PublicProblemConfig, &PrivateUserData)) -> Self {
        Self {
            can_manage: problem.owner_id.is_some_and(|owner_id| owner_id == user.id)
                || user.admin_level == AdminLevel::Owner,
            id: problem.id as i32,
            memory_limit_mb: problem.memory_limit_mb,
            name: problem.name.to_shared_string(),
            owner_id: problem.owner_id.unwrap_or(-1) as i32,
            problem_index: problem.problem_index as i32,
            subgroups: ModelRc::new(VecModel::from(
                problem
                    .subgroups
                    .iter()
                    .map(|x| x.into())
                    .collect::<Vec<AJSubgroup>>(),
            )),
            time_limit_ms: problem.time_limit_ms,
        }
    }
}

impl From<SubgroupVerdict> for AJSubgroupVerdict {
    fn from(subgroup_verdict: SubgroupVerdict) -> Self {
        match subgroup_verdict {
            SubgroupVerdict::Ok => AJSubgroupVerdict::Ok,
            SubgroupVerdict::RuntimeError => AJSubgroupVerdict::RuntimeError,
            SubgroupVerdict::TimeLimitExceeded => AJSubgroupVerdict::TimeLimitExceeded,
            SubgroupVerdict::MemoryLimitExceeded => AJSubgroupVerdict::MemoryLimitExceeded,
            SubgroupVerdict::SecurityError => AJSubgroupVerdict::SecurityError,
            SubgroupVerdict::WrongAnswer => AJSubgroupVerdict::WrongAnswer,
            SubgroupVerdict::PresentationError => AJSubgroupVerdict::PresentationError,
            SubgroupVerdict::Skipped => AJSubgroupVerdict::Skipped,
            SubgroupVerdict::Testing => AJSubgroupVerdict::Testing,
        }
    }
}

impl From<&SubgroupResult> for AJSubgroupResult {
    fn from(subgroup_result: &SubgroupResult) -> Self {
        Self {
            score: subgroup_result.score,
            subgroup_verdict: subgroup_result.subgroup_verdict.clone().into(),
            test: subgroup_result.test,
        }
    }
}

impl From<Language> for AJLanguage {
    fn from(language: Language) -> Self {
        match language {
            Language::Clang => AJLanguage::Clang,
            Language::Clangpp => AJLanguage::Clangpp,
            Language::Rust => AJLanguage::Rust,
            Language::Go => AJLanguage::Go,
            Language::Unknown => AJLanguage::Unknown,
        }
    }
}

impl From<AJLanguage> for Language {
    fn from(language: AJLanguage) -> Self {
        match language {
            AJLanguage::Clang => Language::Clang,
            AJLanguage::Clangpp => Language::Clangpp,
            AJLanguage::Rust => Language::Rust,
            AJLanguage::Go => Language::Go,
            AJLanguage::Unknown => Language::Unknown,
        }
    }
}

impl From<TotalVerdict> for AJTotalVerdict {
    fn from(total_verdict: TotalVerdict) -> Self {
        match total_verdict {
            TotalVerdict::Ok => AJTotalVerdict::Ok,
            TotalVerdict::PartialSolution => AJTotalVerdict::PartialSolution,
            TotalVerdict::Pending => AJTotalVerdict::Pending,
            TotalVerdict::Compiling => AJTotalVerdict::Compiling,
            TotalVerdict::CompilationError => AJTotalVerdict::CompilationError,
            TotalVerdict::Testing => AJTotalVerdict::Testing,
            TotalVerdict::InvalidProblem => AJTotalVerdict::InvalidProblem,
            TotalVerdict::InvalidRequest => AJTotalVerdict::InvalidRequest,
            TotalVerdict::Bug => AJTotalVerdict::Bug,
        }
    }
}

impl From<&Submission> for AJSubmission {
    fn from(submission: &Submission) -> Self {
        Self {
            id: submission.id as i32,
            language: submission.language.clone().into(),
            problem_id: submission.problem_id as i32,
            subgroups_results: ModelRc::new(VecModel::from(
                submission
                    .subgroups_results
                    .iter()
                    .map(|x| x.into())
                    .collect::<Vec<AJSubgroupResult>>(),
            )),
            total_score: submission.total_score,
            total_verdict: submission.total_verdict.clone().into(),
        }
    }
}

impl From<(String, &LeaderboardRow)> for AJLeaderboardRow {
    fn from((username, row): (String, &LeaderboardRow)) -> Self {
        Self {
            user_id: row.user_id as i32,
            scores: ModelRc::new(VecModel::from(row.scores.clone())),
            total_score: row.total_score as i32,
            username: username.to_shared_string(),
        }
    }
}

impl From<AdminLevel> for AJAdminLevel {
    fn from(admin_level: AdminLevel) -> Self {
        match admin_level {
            AdminLevel::NotAdmin => AJAdminLevel::NotAdmin,
            AdminLevel::BetaTester => AJAdminLevel::BetaTester,
            AdminLevel::Admin => AJAdminLevel::Admin,
            AdminLevel::Owner => AJAdminLevel::Owner,
        }
    }
}

impl From<PrivateUserData> for AJAccountData {
    fn from(user: PrivateUserData) -> Self {
        Self {
            admin_level: user.admin_level.into(),
            created_at: user.created_at.into(),
            id: user.id as i32,
            login: user.login.to_shared_string(),
        }
    }
}
