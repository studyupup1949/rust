use std::path::PathBuf;

use directories::ProjectDirs;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Params;

use crate::error::{AocError, DatabaseError};

mod tables;

fn get_data_path() -> Result<PathBuf, AocError> {
    if let Some(path) = ProjectDirs::from("com", "Itron-al-Lenn", "advent-of-util") {
        Ok(path.data_local_dir().to_path_buf())
    } else {
        Err(AocError::Database(DatabaseError::DataLocation))
    }
}

pub struct AocDatabase {
    pool: Pool<SqliteConnectionManager>,
}

impl AocDatabase {
    pub fn new() -> Result<Self, AocError> {
        let path = get_data_path()?.join("aou.db");

        let manager = SqliteConnectionManager::file(&path);
        let pool = Pool::builder()
            .max_size(10) // Adjust pool size as needed
            .build(manager)
            .map_err(|error| {
                AocError::Database(DatabaseError::ConnectionFailed {
                    path: path.clone(),
                    source: error,
                })
            })?;

        // Initialize database with first connection
        let conn = pool.get().map_err(|error| {
            AocError::Database(DatabaseError::ConnectionFailed {
                path: path.clone(),
                source: error,
            })
        })?;

        if let Err(error) = conn.execute("PRAGMA foreign_keys = ON", []) {
            return Err(AocError::Database(DatabaseError::DatabaseExec {
                command: "PRAGMA foreign_keys = ON".to_string(),
                source: error,
            }));
        }

        let db = Self { pool };

        db.create_inputs()?;
        db.create_results()?;
        db.create_test_results()?;

        Ok(db)
    }

    pub fn execute<P>(&self, sql: &str, params: P) -> Result<usize, AocError>
    where
        P: Params,
    {
        let conn = self.pool.get().map_err(|error| {
            AocError::Database(DatabaseError::ConnectionFailed {
                path: get_data_path().unwrap().join("aou.db3"),
                source: error,
            })
        })?;

        match conn.execute(sql, params) {
            Ok(changed) => Ok(changed),
            Err(error) => Err(AocError::Database(DatabaseError::DatabaseExec {
                command: sql.to_string(),
                source: error,
            })),
        }
    }

    // Helper method to get a connection from the pool
    pub fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, AocError> {
        self.pool.get().map_err(|error| {
            AocError::Database(DatabaseError::ConnectionFailed {
                path: get_data_path().unwrap().join("aou.db3"),
                source: error,
            })
        })
    }
}
