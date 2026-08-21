//! App configurations struct and methods to handle state
//! across all the endpoints handlers, including a database pool.
//!
//! Module only available when the `sqlx-postgres` feature is activated.

use crate::db::Tx;
use crate::result::{AppError, Result};
use log::debug;
use server_env_config::db::DbConfig;
use server_env_config::Config;
use sqlx::postgres::{PgConnection, PgPoolOptions};
use sqlx::{Connection, PgPool};

/// Struct that holds the app configurations and the connection pool to the database.
/// Each API method that needs to connect to the database should receive
/// AppState as argument.
///
/// It also has facility methods to handle transactions
/// (see [`AppState::get_tx()`], [`AppState::commit_tx()`]
/// and  [`AppState::rollback_tx()`]).
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Option<PgPool>,
    pub config: Config,
}

impl AppState {
    /// Receive the configuration and initialize
    /// the app state, creating a pool of connections to the DB.
    /// This method normally is used once at startup time
    /// when configuring the Actix HTTP Server.
    ///
    /// Then in your server code each time [`AppState::get_tx()`]
    /// is called, the TX is created from the pool.
    ///
    /// # Examples
    /// ```
    /// use actix_contrib_rest::app_state::AppState;
    /// use actix_web::middleware::Logger;
    /// use actix_web::{App, HttpServer, web};
    /// use server_env_config::Config;
    ///
    /// async fn init(config: &Config) -> anyhow::Result<()> {
    ///     let state = AppState::init(config.clone()).await;
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
    pub async fn init(config: Config) -> core::result::Result<AppState, String> {
        match Self::create_pool(&config.db) {
            Ok(pool) => {
                // The connection is lazy, so not sure whether the connection will work
                debug!("Connection configuration to the database looks good");
                Ok(AppState { pool: Some(pool), config })
            }
            Err(err) => {
                // Errors like wrongly parsed URLs arrive here, but not errors
                // trying to connect to
                Err(format!("Failed to connect to the database: {:?}", err))
            }
        }
    }

    /// Create an AppState but without a pool initialized.
    ///
    /// This way [`AppState::get_tx()`] cannot be called because there is no pool
    /// to get a transaction from.  Use [`AppState::get_conn()`] instead.
    ///
    /// See [`AppState::init()`] to create the state with a pool of connections.
    pub fn new(config: Config) -> AppState {
        AppState {
            config,
            pool: None,
        }
    }

    /// Get a Transaction object to perform SQL operations with the DB.
    ///
    /// Once used [`AppState::commit_tx()`] should be called to finish and release
    /// the TX, or [`AppState::rollback_tx()`] to release it rolling back the changes
    /// in case of errors.
    ///
    /// The TX is created from a connection within the poll of the AppState, or fails
    /// with a `AppError::StaticValidation("Pool not initialized")` if the state
    /// was not initialized with a pool (see [`AppState::init()`]).
    ///
    /// If the pool is not initialized, to acquire a transaction use [`AppState::get_conn()`]
    /// instead.
    ///
    /// # Examples
    /// ```
    /// use actix_web::{post, HttpResponse};
    /// use actix_web::web::Data;
    /// use actix_contrib_rest::app_state::AppState;
    /// use actix_contrib_rest::result::{AppError, HttpResult};
    /// use actix_web_validator::Json;
    /// use serde::{Deserialize, Serialize};
    /// use validator::Validate;
    ///
    /// #[derive(sqlx::FromRow)]
    /// #[derive(Deserialize, Serialize, Validate)]
    /// pub struct Comment { pub author_id: i64, pub txt: String }
    ///
    /// #[post("/comments")]
    /// async fn create(app: Data<AppState>, comment: Json<Comment>) -> HttpResult {
    ///     let mut tx = app.get_tx().await?;   // Create the TX
    ///     let rec = sqlx::query_as::<_, Comment>(
    ///             "INSERT INTO comments (author_id, txt, created_at) VALUES ($1, $2, NOW()) RETURNING *"
    ///         )
    ///         .bind(comment.author_id)
    ///         .bind(comment.txt.as_str())
    ///         .fetch_one(&mut *tx)            // Depending of sqlx version -> try `&mut **tx` instead
    ///         .await
    ///         .map_err(AppError::DB)?;        // If fails, the TX is rolled back automatically
    ///     app.commit_tx(tx).await?;           // If success, commit and release the TX
    ///     Ok(HttpResponse::Created().json(rec))
    /// }
    /// ```
    pub async fn get_tx(&self) -> Result<Tx<'_>> {
        self.pool
            .as_ref()
            .ok_or_else(|| AppError::StaticValidation("Pool not initialized"))?
            .begin()
            .await
            .map_err(AppError::DB)
    }

    /// Get a connection to the database. Use this method if the pool
    /// has not been initialized and you need a single connection,
    /// otherwise better to use [`AppState::get_tx()`].
    ///
    /// A [`Tx`] can be created with the connection using `Connection::begin` (see example),
    /// and once used, [`AppState::commit_tx()`] should be called to finish and release it,
    /// or [`AppState::rollback_tx()`] to release it rolling back the changes
    /// in case of errors.
    ///
    /// # Example
    /// ```
    /// use actix_contrib_rest::app_state::AppState;
    /// use actix_contrib_rest::result::{AppError, Result};
    /// use sqlx::Connection;
    ///
    /// async fn count_comments(state: AppState) -> Result<i64> {
    ///     let mut conn = state.get_conn().await?; // Open connection
    ///     let mut tx = Connection::begin(&mut conn).await.map_err(AppError::DB)?; // Create transaction
    ///     let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comments")
    ///             .fetch_one(&mut *tx)            // Use Tx to perform SQL operation
    ///             .await
    ///             .map_err(AppError::DB)?;
    ///     state.commit_tx(tx).await?;             // Commit, closing the Tx
    ///     Ok(count.0)
    /// }
    /// ```
    pub async fn get_conn(&self) -> Result<PgConnection> {
        let conn = PgConnection::connect(self.config.db.database_url.as_ref())
            .await
            .map_err(AppError::DB);
        conn
    }

    /// Commit the transaction passed. The method
    /// takes ownership of the TX, making it not usable
    /// anymore.
    ///
    /// To rollback instead, see [`AppState::rollback_tx()`].
    ///
    /// See also [`AppState::get_tx()`] and [`AppState::get_conn()`].
    pub async fn commit_tx(&self, tx: Tx<'_>) -> Result<()> {
        tx.commit().await.map_err(AppError::DB)?;
        Ok(())
    }

    /// Rollback the transaction passed. The method
    /// takes ownership of the TX, making it not usable
    /// anymore.
    ///
    /// To commit instead, see [`AppState::commit_tx()`].
    ///
    /// See [`AppState::get_tx()`] and [`AppState::get_conn()`].
    #[allow(dead_code)]
    pub async fn rollback_tx(&self, tx: Tx<'_>) -> Result<()> {
        tx.rollback().await.map_err(AppError::DB)?;
        Ok(())
    }

    /// Create a pool of connections.
    ///
    /// This method is called internally by [`AppState::init()`],
    /// so there is no need to call it again to acquire a connection,
    /// in which case [`AppState::get_tx()`] should be used.
    ///
    /// For a single connection better to use [`AppState::get_conn()`].
    pub fn create_pool(config: &DbConfig) -> Result<PgPool> {
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
