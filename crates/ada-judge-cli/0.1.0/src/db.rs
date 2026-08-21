use ada_judge_public_models::problems::{ProblemConfig, Subgroup};
use anyhow::Ok;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::config::Config;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            pool: PgPool::connect(&config.db_url).await?,
        })
    }

    pub async fn insert_problem(&self, problem_config: &ProblemConfig) -> anyhow::Result<i64> {
        let problem_id: i64 = sqlx::query_scalar(
            "insert into problems (owner_id, contest_id, problem_index, name, time_limit_ms, memory_limit_mb, checker_path, tests_path)
                    values
                        ($1, $2, $3, $4, $5, $6, $7, $8)
                    returning id")
                .bind(problem_config.owner_id)
                .bind(problem_config.contest_id)
                .bind(problem_config.problem_index)
                .bind(problem_config.name.clone())
                .bind(problem_config.time_limit_ms)
                .bind(problem_config.memory_limit_mb)
                .bind(problem_config.checker_path.clone())
                .bind(problem_config.tests_path.clone())
                .fetch_one(&self.pool)
                .await?;

        Ok(problem_id)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub async fn insert_problems_subgroup(
        &self,
        problem_id: i64,
        subgroup: &Subgroup,
        ind: i64,
    ) -> anyhow::Result<()> {
        sqlx::query("insert into problems_subgroups (problem_id, subgroup_index, type, tests, score, depends_on)
                            values
                                ($1, $2, $3, $4, $5, $6)")
        .bind(problem_id)
        .bind(ind)
        .bind(subgroup.r#type.clone())
        .bind(subgroup.tests.clone())
        .bind(subgroup.score)
        .bind(subgroup.depends_on.iter().map(|x| *x as i32).collect::<Vec<i32>>())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_contest(
        &self,
        owner_id: Option<&i64>,
        name: &str,
        starts_at: &DateTime<Utc>,
        ends_at: &DateTime<Utc>,
    ) -> anyhow::Result<i64> {
        let contest_id: i64 = sqlx::query_scalar(
            "insert into contests (owner_id, name, starts_at, ends_at)
                                    values ($1, $2, $3, $4) returning id",
        )
        .bind(owner_id)
        .bind(name)
        .bind(starts_at)
        .bind(ends_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(contest_id)
    }
}
