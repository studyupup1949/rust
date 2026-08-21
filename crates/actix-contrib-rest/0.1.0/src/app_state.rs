//! App configurations struct and methods to handle state
//! across all the endpoints handlers, including a database pool.
//!
//! Module only available when the `sqlx-postgres` feature is activated.

use crate::db::Tx;
use crate::result::{AppError, Result};
use log::debug;
use server_env_config::db::DbConfig;
use server_env_config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Struct that holds the app configurations and the connection pool to the database.
/// Each API method that needs to connect to the database should receive
/// AppState as argument.
///
/// It also has facility methods to handle transactions,
/// (see [`AppState::get_tx()`], [`AppState::commit_tx()`]
/// and  [`AppState::rollback_tx()`]).
/// and the [`AppState::new()`] method to initialize
/// the structure, like the connection pool with the DB.
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

impl AppState {
    /// Receive the configuration and initialize
    /// the app state, like the pool connection to the DB.
    /// This method normally is used once at startup time
    /// when configuring the Actix HTTP Server.
    /// # Examples
    /// ```
    /// use actix_contrib_rest::app_state::AppState;
    /// use actix_web::middleware::Logger;
    /// use actix_web::{App, HttpServer, web};
    /// use server_env_config::Config;
    ///
    /// async fn init(config: &Config) -> anyhow::Result<()> {
    ///     let state = AppState::new(config.clone()).await;
    ///     let server = HttpServer::new(move || {
    ///         App::new()
    ///            .app_data(web::Data::new(state.clone()))
    ///            //.configure(comments_api::config).configure(... api handlers)...
    ///            .wrap(Logger::default())
    ///     })
    ///     .bind((config.server.addr.clone(), config.server.port))?
    ///     .run();
    ///     Ok(())
    /// }
    /// // ...
    /// ```
    pub async fn new(config: Config) -> core::result::Result<AppState, String> {
        match Self::_get_pool(&config.db) {
            Ok(pool) => {
                // The connection is lazy, so not sure whether the connection will work
                debug!("Connection configuration to the database looks good");
                Ok(AppState { pool, config })
            }
            Err(err) => {
                // Errors like wrongly parsed URLs arrive here, but not errors
                // trying to connect to
                Err(format!("Failed to connect to the database: {:?}", err))
            }
        }
    }

    /// Get a Transaction object to be used to transact with the DB.
    /// Once used [`AppState::commit_tx()`] should be used to finish
    /// and release the TX, or [`AppState::rollback_tx()`] to
    /// release it rolling back the changes.
    /// # Examples
    /// ```
    /// use actix_web::{post, HttpResponse};
    /// use actix_web::web::Data;
    /// use actix_contrib_rest::app_state::AppState;
    /// use actix_web_validator::Json;
    ///
    /// #[post("/comments")]
    /// async fn create(app: Data<AppState>, comment: Json<CommentPayload>) -> HttpResult {
    ///     let mut tx = app.get_tx().await?;   // Create the TX
    ///     let rec = sqlx::query_as!(
    ///             Comment,
    ///             "INSERT INTO comments (txt, ..., created_at) VALUES ($1, ..., NOW()) RETURNING *",
    ///             comment.txt, ...)
    ///         .fetch_one(&mut **tx)           // Pass the TX to as many ops as needed
    ///         .await
    ///         .map_err(AppError::DB)?;        // If fails, the TX is rolled back automatically
    ///     app.commit_tx(tx).await?;           // If success, commit and release the TX
    ///     Ok(HttpResponse::Created().json(rec))
    /// }
    /// ```
    pub async fn get_tx(&self) -> Result<Tx<'_>> {
        self.pool.begin().await.map_err(AppError::DB)
    }

    /// Commit the transaction passed. The method
    /// takes ownership of the TX, making it not usable
    /// anymore.
    /// See [`AppState::get_tx()`].
    pub async fn commit_tx(&self, tx: Tx<'_>) -> Result<()> {
        tx.commit().await.map_err(AppError::DB)?;
        Ok(())
    }

    /// Rollback the transaction passed. The method
    /// takes ownership of the TX, making it not usable
    /// anymore.
    /// See [`AppState::get_tx()`].
    #[allow(dead_code)]
    pub async fn rollback_tx(&self, tx: Tx<'_>) -> Result<()> {
        tx.rollback().await.map_err(AppError::DB)?;
        Ok(())
    }

    fn _get_pool(config: &DbConfig) -> Result<PgPool> {
        PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .test_before_acquire(config.test_before_acquire)
            .connect_lazy(&config.database_url)
            .map_err(AppError::DB)
    }
}
