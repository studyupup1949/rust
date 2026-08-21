//! Connection pool for TDengine.
//!
//! Provides efficient connection pooling using deadpool for better resource management
//! when multiple connections are needed.

use std::sync::Arc;

use async_trait::async_trait;
use deadpool::managed::{Manager, Metrics, Object, Pool, RecycleError};
use taos_client::AsyncQueryable;
use taos_client::AsyncTBuilder;

use super::error::TaosError;
use super::utils::Runtime;
use taos_client::Taos;
use taos_client::TaosBuilder;

/// Configuration for TDengine connection pool.
#[derive(Debug, Clone)]
pub struct TaosPoolConfig {
    /// DSN for connecting to TDengine
    pub dsn: String,
    /// Maximum pool size (default: 16)
    pub max_size: usize,
}

impl Default for TaosPoolConfig {
    fn default() -> Self {
        Self {
            dsn: "taos://root:taosdata@127.0.0.1:6030".to_string(),
            max_size: 16,
        }
    }
}

/// Pooled TDengine connection wrapper.
///
/// This type wraps a deadpool managed object that contains a TDengine connection.
/// When dropped, the connection is returned to the pool instead of being closed.
pub type PooledTaosConnection = Object<TaosPoolManager>;

/// Internal wrapper for TDengine connection.
///
/// Holds the connection along with runtime metadata.
/// TDengine connections are Arc-wrapped and threadsafe.
#[derive(Debug)]
pub struct TaosConnectionWrapper {
    /// Async runtime for bridging async TDengine client
    rt: Arc<Runtime>,
    /// TDengine connection
    conn: Arc<Taos>,
}

impl TaosConnectionWrapper {
    /// Returns a reference to the inner TDengine connection.
    pub fn conn(&self) -> &Taos {
        &self.conn
    }

    /// Returns a clone of the Arc wrapping the connection.
    pub fn conn_clone(&self) -> Arc<Taos> {
        Arc::clone(&self.conn)
    }

    /// Returns a reference to the runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.rt
    }
}

/// Manager for TDengine connection pool.
///
/// Implements deadpool's Manager trait to handle connection lifecycle:
/// - Creating new connections
/// - Recycling existing connections (health checks)
/// - Checking connection validity
#[derive(Debug)]
pub struct TaosPoolManager {
    /// DSN for connection
    dsn: String,
    /// Runtime for async operations
    rt: Arc<Runtime>,
}

impl TaosPoolManager {
    /// Creates a new pool manager.
    ///
    /// # Arguments
    /// * `dsn` - TDengine DSN string
    /// * `rt` - Async runtime for bridging
    pub fn new(dsn: String, rt: Arc<Runtime>) -> Self {
        Self { dsn, rt }
    }
}

#[async_trait]
impl Manager for TaosPoolManager {
    type Type = TaosConnectionWrapper;
    type Error = TaosError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let dsn = self.dsn.clone();
        let builder = TaosBuilder::from_dsn(&dsn)
            .map_err(|e| TaosError::connection(format!("Failed to parse DSN: {}", e)))?;
        let conn = builder.build().await
            .map_err(|e| TaosError::connection(format!("Failed to connect: {}", e)))?;

        Ok(TaosConnectionWrapper {
            rt: self.rt.clone(),
            conn: Arc::new(conn),
        })
    }

    async fn recycle(&self, conn: &mut Self::Type, _: &Metrics) -> Result<(), RecycleError<Self::Error>> {
        // Health check: verify server is still responsive
        let conn_ref = Arc::clone(&conn.conn);
        conn_ref.server_version().await
            .map_err(|e| RecycleError::Backend(TaosError::connection(format!(
                "Connection health check failed: {}",
                e
            ))))?;
        Ok(())
    }
}

/// TDengine connection pool.
///
/// Provides a pool of reusable TDengine connections for efficient resource management.
/// Connections are automatically recycled and validated on reuse.
///
/// # Example
/// ```ignore
/// let pool = TaosPool::new(dsn, rt)?;
/// let mut conn = pool.get().await?;
/// // Use connection...
/// // Connection is returned to pool when dropped
/// ```
#[derive(Clone)]
pub struct TaosPool {
    /// Inner deadpool
    pool: Pool<TaosPoolManager>,
}

impl std::fmt::Debug for TaosPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaosPool")
            .field("status", &self.pool.status())
            .finish()
    }
}

impl TaosPool {
    /// Creates a new connection pool.
    ///
    /// # Arguments
    /// * `dsn` - TDengine DSN connection string
    /// * `rt` - Async runtime for bridging
    ///
    /// # Errors
    /// Returns error if pool configuration fails
    pub fn new(dsn: String, rt: Arc<Runtime>) -> Result<Self, deadpool::managed::BuildError> {
        Self::with_config(TaosPoolConfig { dsn, ..Default::default() }, rt)
    }

    /// Creates a new connection pool with custom configuration.
    ///
    /// # Arguments
    /// * `config` - Pool configuration
    /// * `rt` - Async runtime for bridging
    ///
    /// # Errors
    /// Returns error if pool configuration fails
    pub fn with_config(
        config: TaosPoolConfig,
        rt: Arc<Runtime>,
    ) -> Result<Self, deadpool::managed::BuildError> {
        let manager = TaosPoolManager::new(config.dsn, rt);

        let pool = Pool::builder(manager)
            .max_size(config.max_size)
            .build()?;

        Ok(Self { pool })
    }

    /// Gets a connection from the pool asynchronously.
    ///
    /// # Errors
    /// Returns error if no connection is available or connection creation fails
    pub async fn get(&self) -> Result<PooledTaosConnection, TaosError> {
        self.pool
            .get()
            .await
            .map_err(|e| match e {
                deadpool::managed::PoolError::Backend(e) => e,
                deadpool::managed::PoolError::Timeout(e) => {
                    TaosError::connection(format!("Pool timeout: {:?}", e))
                }
                deadpool::managed::PoolError::Closed => {
                    TaosError::connection("Pool closed".to_string())
                }
                deadpool::managed::PoolError::NoRuntimeSpecified => {
                    TaosError::connection("No runtime specified".to_string())
                }
                deadpool::managed::PoolError::PostCreateHook(_) => {
                    TaosError::connection("Post create hook failed".to_string())
                }
            })
    }

    /// Returns pool status information.
    pub fn status(&self) -> deadpool::Status {
        self.pool.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = TaosPoolConfig::default();
        assert_eq!(config.max_size, 16);
        assert_eq!(
            config.dsn,
            "taos://root:taosdata@127.0.0.1:6030"
        );
    }

    #[test]
    fn test_pool_config_clone() {
        let config = TaosPoolConfig {
            dsn: "taos://test:test@localhost:6030".to_string(),
            max_size: 32,
        };
        let cloned = config.clone();
        assert_eq!(cloned.dsn, config.dsn);
        assert_eq!(cloned.max_size, config.max_size);
    }
}

// Rust guideline compliant 2025-01-02
