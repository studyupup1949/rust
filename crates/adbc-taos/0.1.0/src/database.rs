//! Database implementation for ADBC-Taos driver.
//!
//! The `TaosDatabase` holds connection configuration and creates connections
//! to TDengine.

use std::sync::Arc;

use adbc_core::{Database, Optionable, options::{OptionDatabase, OptionValue}};
use taos_client::AsyncTBuilder;
use taos_client::TaosBuilder;

use super::connection::TaosConnection;
use super::error::TaosError;
use super::pool::{TaosPool, TaosPoolConfig};
use super::utils::Runtime;

/// Database configuration holder.
///
/// Stores connection parameters (URI, user, password, database name)
/// and creates TDengine connections with optional connection pooling.
pub struct TaosDatabase {
    /// TDengine DSN (Data Source Name)
    pub uri: String,
    /// Username for authentication
    pub user: String,
    /// Password for authentication
    pub password: String,
    /// Maximum pool size (0 = no pooling, create direct connections)
    pool_size: usize,
    /// Internal connection pool (created when pool_size > 0)
    pool: Option<PoolState>,
}

/// Internal pool state holding the runtime and pool.
struct PoolState {
    /// Async runtime for blocking on async pool operations
    rt: Arc<Runtime>,
    /// Connection pool
    pool: TaosPool,
}

impl Default for TaosDatabase {
    fn default() -> Self {
        Self {
            uri: "taos://127.0.0.1:6030".to_string(),
            user: "root".to_string(),
            password: "taosdata".to_string(),
            pool_size: 0,
            pool: None,
        }
    }
}

impl TaosDatabase {
    /// Sets the connection pool size and initializes the pool.
    ///
    /// # Arguments
    /// * `size` - Maximum number of connections in the pool (0 = no pooling)
    ///
    /// # Example
    /// ```ignore
    /// let mut db = TaosDatabase::default();
    /// db.set_pool_size(10);
    /// let conn = db.new_connection()?;
    /// ```
    pub fn set_pool_size(&mut self, size: usize) {
        self.pool_size = size;
        self.pool = None;
    }

    /// Returns the current pool size.
    pub fn pool_size(&self) -> usize {
        self.pool_size
    }

    /// Initializes the connection pool if not already initialized.
    fn ensure_pool(&mut self) -> adbc_core::error::Result<()> {
        if self.pool_size > 0 && self.pool.is_none() {
            let dsn = build_dsn(&self.uri, &self.user, &self.password)
                .map_err(|e| adbc_core::error::Error::with_message_and_status(
                    e.to_string(),
                    e.adbc_status(),
                ))?;

            let rt = Arc::new(
                Runtime::new()
                    .map_err(|e| TaosError::connection(format!("Failed to create runtime: {}", e)))?,
            );

            let pool_config = TaosPoolConfig {
                dsn,
                max_size: self.pool_size,
            };

            let pool = TaosPool::with_config(pool_config, rt.clone())
                .map_err(|e| adbc_core::error::Error::with_message_and_status(
                    format!("Failed to create pool: {}", e),
                    adbc_core::error::Status::Internal,
                ))?;

            self.pool = Some(PoolState { rt, pool });
        }
        Ok(())
    }

    /// Creates a new connection, using pool if configured.
    ///
    /// This method requires `&mut self` to ensure pool is initialized before
    /// creating connections. For ADBC compatibility, prefer using the
    /// `Database::new_connection()` trait method after setting pool_size.
    pub fn create_connection(&mut self) -> adbc_core::error::Result<TaosConnection> {
        self.ensure_pool()?;
        if let Some(pool_state) = &self.pool {
            let wrapper = pool_state.rt.block_on(pool_state.pool.get())
                .map_err(|e| adbc_core::error::Error::with_message_and_status(
                    format!("Failed to get connection from pool: {}", e),
                    adbc_core::error::Status::Internal,
                ))?;
            return Ok(TaosConnection::new(pool_state.rt.clone(), wrapper.conn_clone()));
        }
        self.create_direct_connection()
    }

    /// Creates a direct connection without pooling.
    fn create_direct_connection(&self) -> adbc_core::error::Result<TaosConnection> {
        let dsn = build_dsn(&self.uri, &self.user, &self.password)
            .map_err(|e| adbc_core::error::Error::with_message_and_status(e.to_string(), e.adbc_status()))?;

        let rt = Arc::new(
            Runtime::new()
                .map_err(|e| TaosError::connection(format!("Failed to create runtime: {}", e)))?,
        );

        let conn = rt
            .block_on(async {
                let builder = TaosBuilder::from_dsn(&dsn)?;
                builder.build().await
            })
            .map_err(|e| TaosError::connection(format!("Failed to connect: {}", e)))?;

        Ok(TaosConnection::new(rt, Arc::new(conn)))
    }
}

impl Optionable for TaosDatabase {
    type Option = OptionDatabase;

    fn set_option(&mut self, key: Self::Option, value: OptionValue) -> adbc_core::error::Result<()> {
        let value = match value {
            OptionValue::String(value) => value,
            _ => {
                return Err(adbc_core::error::Error::with_message_and_status(
                    "Expected string value for database option",
                    adbc_core::error::Status::InvalidArguments,
                ));
            }
        };
        match key {
            OptionDatabase::Uri => self.uri = value,
            OptionDatabase::Username => self.user = value,
            OptionDatabase::Password => self.password = value,
            _ => {
                return Err(adbc_core::error::Error::with_message_and_status(
                    "Unsupported database option",
                    adbc_core::error::Status::NotImplemented,
                ));
            }
        }
        Ok(())
    }

    fn get_option_string(&self, key: Self::Option) -> adbc_core::error::Result<String> {
        match key {
            OptionDatabase::Uri => Ok(self.uri.clone()),
            OptionDatabase::Username => Ok(self.user.clone()),
            OptionDatabase::Password => Ok(self.password.clone()),
            _ => Err(adbc_core::error::Error::with_message_and_status(
                "Unsupported database option",
                adbc_core::error::Status::NotImplemented,
            )),
        }
    }

    fn get_option_bytes(&self, _key: Self::Option) -> adbc_core::error::Result<Vec<u8>> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Unsupported database option",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_double(&self, _key: Self::Option) -> adbc_core::error::Result<f64> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Unsupported database option",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_int(&self, _key: Self::Option) -> adbc_core::error::Result<i64> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Unsupported database option",
            adbc_core::error::Status::NotImplemented,
        ))
    }
}

impl Database for TaosDatabase {
    type ConnectionType = TaosConnection;

    fn new_connection(&self) -> adbc_core::error::Result<Self::ConnectionType> {
        self.new_connection_with_opts(std::iter::empty())
    }

    fn new_connection_with_opts(
        &self,
        opts: impl IntoIterator<Item = (adbc_core::options::OptionConnection, OptionValue)>,
    ) -> adbc_core::error::Result<Self::ConnectionType> {
        // If pool is initialized, use pooled connection
        if let Some(pool_state) = &self.pool {
            let wrapper = pool_state.rt.block_on(pool_state.pool.get())
                .map_err(|e| adbc_core::error::Error::with_message_and_status(
                    format!("Failed to get connection from pool: {}", e),
                    adbc_core::error::Status::Internal,
                ))?;
            return Ok(TaosConnection::new(pool_state.rt.clone(), wrapper.conn_clone()));
        }

        // Direct connection path (no pooling)
        let dsn = build_dsn(&self.uri, &self.user, &self.password)
            .map_err(|e| adbc_core::error::Error::with_message_and_status(e.to_string(), e.adbc_status()))?;

        let rt = Arc::new(
            Runtime::new()
                .map_err(|e| TaosError::connection(format!("Failed to create runtime: {}", e)))?,
        );

        let conn = rt
            .block_on(async {
                let builder = TaosBuilder::from_dsn(&dsn)?;
                for (_key, _value) in opts {
                    // TODO: Apply connection-specific options
                }
                builder.build().await
            })
            .map_err(|e| TaosError::connection(format!("Failed to connect: {}", e)))?;

        Ok(TaosConnection::new(rt, Arc::new(conn)))
    }
}

/// Builds a TDengine DSN from URI components.
fn build_dsn(
    uri: &str,
    user: &str,
    password: &str,
    // _database: Option<&str>,
) -> std::result::Result<String, TaosError> {
    // Parse URI format: taos://host:port or taos://user:pass@host:port/db
    let parts: Vec<&str> = uri.split("://").collect();
    if parts.len() < 2 {
        return Err(TaosError::invalid_option("Invalid URI format".to_string()));
    }

    let after_protocol = parts[1];
    let has_at = after_protocol.contains('@');

    let (final_user, final_pass, host_port) = if has_at {
        let creds_part = after_protocol.split('@').next().unwrap_or("");
        let rest = after_protocol.split('@').nth(1).unwrap_or("");

        let uri_user = creds_part.split(':').next().unwrap_or(user);
        let uri_pass = creds_part.split(':').nth(1).unwrap_or(password);

        (uri_user, uri_pass, rest.split('/').next().unwrap_or(rest))
    } else {
        (user, password, after_protocol.split('/').next().unwrap_or(after_protocol))
    };

    // URL-encode credentials to handle special characters like #, @, :, etc.
    let encoded_user = urlencoding::encode(final_user);
    let encoded_pass = urlencoding::encode(final_pass);
    let dsn = format!("taos://{}:{}@{}", encoded_user, encoded_pass, host_port);
    Ok(dsn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use adbc_core::options::OptionDatabase;

    #[test]
    fn test_default_database() {
        let db = TaosDatabase::default();
        assert_eq!(db.uri, "taos://127.0.0.1:6030");
        assert_eq!(db.user, "root");
        assert_eq!(db.password, "taosdata");
    }

    #[test]
    fn test_build_dsn_simple() {
        let dsn = build_dsn("taos://127.0.0.1:6030", "root", "taosdata").unwrap();
        assert_eq!(dsn, "taos://root:taosdata@127.0.0.1:6030");
    }

    #[test]
    fn test_build_dsn_with_credentials_in_uri() {
        let dsn = build_dsn("taos://admin:secret@192.168.1.1:6030", "root", "taosdata").unwrap();
        assert_eq!(dsn, "taos://admin:secret@192.168.1.1:6030");
    }

    #[test]
    fn test_build_dsn_special_characters() {
        let dsn = build_dsn("taos://127.0.0.1:6030", "root", "pass#word@123").unwrap();
        assert!(dsn.contains("pass%23word%40123"));
    }

    #[test]
    fn test_build_dsn_invalid_uri() {
        let result = build_dsn("invalid_uri", "root", "taosdata");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_option_uri() {
        let mut db = TaosDatabase::default();
        db.set_option(OptionDatabase::Uri, OptionValue::String("taos://192.168.1.1:6030".to_string())).unwrap();
        assert_eq!(db.uri, "taos://192.168.1.1:6030");
    }

    #[test]
    fn test_set_option_username() {
        let mut db = TaosDatabase::default();
        db.set_option(OptionDatabase::Username, OptionValue::String("admin".to_string())).unwrap();
        assert_eq!(db.user, "admin");
    }

    #[test]
    fn test_set_option_password() {
        let mut db = TaosDatabase::default();
        db.set_option(OptionDatabase::Password, OptionValue::String("secret".to_string())).unwrap();
        assert_eq!(db.password, "secret");
    }

    #[test]
    fn test_get_option_string() {
        let db = TaosDatabase::default();
        assert_eq!(db.get_option_string(OptionDatabase::Uri).unwrap(), "taos://127.0.0.1:6030");
        assert_eq!(db.get_option_string(OptionDatabase::Username).unwrap(), "root");
        assert_eq!(db.get_option_string(OptionDatabase::Password).unwrap(), "taosdata");
    }

    #[test]
    fn test_pool_size_default() {
        let db = TaosDatabase::default();
        assert_eq!(db.pool_size(), 0);
    }

    #[test]
    fn test_set_pool_size() {
        let mut db = TaosDatabase::default();
        db.set_pool_size(10);
        assert_eq!(db.pool_size(), 10);
    }
}
