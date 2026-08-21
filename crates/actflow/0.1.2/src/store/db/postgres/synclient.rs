use std::{sync::Arc, time::Duration};

use sqlx::{
    Database, Error, IntoArguments, PgPool, Postgres,
    postgres::{PgPoolOptions, PgRow},
};
use tokio::{
    runtime::{Handle, Runtime},
    task::block_in_place,
};

#[derive(Debug, Clone)]
pub struct SynClient {
    pool: PgPool,

    runtime: Arc<Runtime>,
}

impl SynClient {
    pub fn connect(
        db_url: &str,
        runtime: Arc<Runtime>,
    ) -> Self {
        #[allow(clippy::expect_fun_call)]
        let pool = if Handle::try_current().is_ok() {
            block_in_place(|| {
                runtime.block_on(async move { PgPoolOptions::new().acquire_timeout(Duration::from_secs(5)).max_connections(200).connect(db_url).await })
            })
        } else {
            runtime.block_on(async move { PgPoolOptions::new().acquire_timeout(Duration::from_secs(5)).max_connections(200).connect(db_url).await })
        }
        .expect(&format!("failed to connect to DB {}", db_url));

        Self {
            pool,
            runtime,
        }
    }

    pub fn query_one<'q, A>(
        &self,
        sql: &'q str,
        params: A,
    ) -> Result<PgRow, Error>
    where
        A: IntoArguments<'q, Postgres> + 'q,
    {
        if Handle::try_current().is_ok() {
            block_in_place(|| {
                self.runtime.block_on(async move {
                    let mut conn = self.pool.acquire().await?;

                    sqlx::query_with(sql, params).fetch_one(&mut *conn).await
                })
            })
        } else {
            self.runtime.block_on(async move {
                let mut conn = self.pool.acquire().await?;

                sqlx::query_with(sql, params).fetch_one(&mut *conn).await
            })
        }
    }

    pub fn query<'q, A>(
        &self,
        sql: &'q str,
        params: A,
    ) -> Result<Vec<PgRow>, Error>
    where
        A: IntoArguments<'q, Postgres> + 'q,
    {
        if Handle::try_current().is_ok() {
            block_in_place(|| {
                self.runtime.block_on(async move {
                    let mut conn = self.pool.acquire().await?;

                    sqlx::query_with(sql, params).fetch_all(&mut *conn).await
                })
            })
        } else {
            self.runtime.block_on(async move {
                let mut conn = self.pool.acquire().await?;

                sqlx::query_with(sql, params).fetch_all(&mut *conn).await
            })
        }
    }

    pub fn execute<'q, A>(
        &self,
        sql: &'q str,
        params: A,
    ) -> Result<<Postgres as Database>::QueryResult, Error>
    where
        A: IntoArguments<'q, Postgres> + 'q,
    {
        if Handle::try_current().is_ok() {
            block_in_place(|| {
                self.runtime.block_on(async move {
                    let mut conn = self.pool.acquire().await?;

                    sqlx::query_with(sql, params).execute(&mut *conn).await
                })
            })
        } else {
            self.runtime.block_on(async move {
                let mut conn = self.pool.acquire().await?;

                sqlx::query_with(sql, params).execute(&mut *conn).await
            })
        }
    }

    pub fn batch_execute(
        &self,
        sqls: &[String],
    ) -> Result<(), Error> {
        if Handle::try_current().is_ok() {
            block_in_place(|| {
                self.runtime.block_on(async move {
                    let mut tx = self.pool.begin().await?;

                    for sql in sqls {
                        sqlx::query(sql).execute(&mut *tx).await?;
                    }
                    tx.commit().await
                })
            })
        } else {
            self.runtime.block_on(async move {
                let mut tx = self.pool.begin().await?;

                for sql in sqls {
                    sqlx::query(sql).execute(&mut *tx).await?;
                }
                tx.commit().await
            })
        }
    }
}
