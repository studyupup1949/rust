//! Database connection facilities.

use activitystreams_vocabulary::field_access;
use sqlx::pool::{Pool, PoolOptions};
use sqlx::{Database, Postgres};

pub use sqlx::types::Uuid;

use crate::{Error, Result, util};

pub mod activity;
pub mod actor;
pub mod crypto;
pub mod object;

mod config;
mod iri;
mod macros;
mod name;
mod table;
mod vocabulary;

pub use activity::*;
pub use actor::*;
pub use config::DbConfig;
pub use crypto::*;
pub use iri::*;
pub use name::*;
pub use object::*;
pub use table::*;
pub use vocabulary::*;

/// Represents a database connection.
pub struct Db<DB: Database = Postgres> {
    config: DbConfig,
    pool: Option<Pool<DB>>,
}

impl<DB: Database> Db<DB> {
    /// Creates a new [Db].
    pub fn new() -> Self {
        Self {
            config: DbConfig::new(),
            pool: None,
        }
    }

    /// Gets a symmetric used for database cryptographic functions.
    pub fn key(&self) -> Result<SymmetricKey> {
        SymmetricKey::derive(self.config.password().as_bytes(), self.salt())
    }

    /// Gets a salt used for database cryptographic functions.
    pub(crate) fn salt(&self) -> Salt {
        // hash the config JSON serialization without the `password` field.
        Salt::hash(self.config.to_string().as_bytes())
    }

    /// Gets a reference to the [Db] connection pool.
    #[inline]
    pub fn pool(&self) -> Result<&Pool<DB>> {
        self.pool
            .as_ref()
            .ok_or(Error::sql("unset database connection"))
    }

    /// Sets the [Db] connection pool.
    pub fn set_pool(&mut self, pool: Pool<DB>) {
        self.pool = Some(pool);
    }

    /// Builder function that sets the [Db] connection pool.
    pub fn with_pool(self, pool: Pool<DB>) -> Self {
        Self {
            pool: Some(pool),
            ..self
        }
    }

    /// Creates a random V4 [Uuid].
    pub fn rand_uuid(&self) -> Uuid {
        util::rand_uuid()
    }
}

impl Db<Postgres> {
    /// Creates a database connection using the supplied configuration settings.
    ///
    /// ```rust,no_run
    /// use activityforge::{Result, db::{Db, DbConfig}};
    ///
    /// # async fn test() -> Result<()> {
    /// let username = "db_user";
    /// let password = "db_password123";
    /// let db_name = "test_db";
    /// let host = "localhost";
    /// let port = 5432u16;
    ///
    /// let config = DbConfig::new()
    ///     .with_username(username)
    ///     .with_password(password)
    ///     .with_db_name(db_name)
    ///     .with_host(host)
    ///     .with_port(port);
    ///
    /// let _db = Db::connect(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(config: DbConfig) -> Result<Self> {
        let conn_str = config.postgres_connection();

        PoolOptions::<Postgres>::new()
            .max_connections(config.max_connections())
            .connect(conn_str.as_str())
            .await
            .map_err(Error::from)
            .map(|pool| Self {
                config,
                pool: Some(pool),
            })
    }
}

field_access! {
    Db {
        /// [Db] configuration settings.
        config: as_ref { DbConfig },
    }
}

impl<DB: Database> Default for Db<DB> {
    fn default() -> Self {
        Self::new()
    }
}
