use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error(
        "Failed executing following command on the Database: \n\"{command}\"\nCaused by: {source}"
    )]
    DatabaseExec {
        command: String,
        #[source]
        source: rusqlite::Error,
    },

    #[error("Failed finding the correct location for local data")]
    DataLocation,

    #[error("Failed connecting to the database at {path}")]
    ConnectionFailed {
        path: PathBuf,
        #[source]
        source: r2d2::Error,
    },

    #[error("Failed querring the database, trying getting {object}: {source}")]
    DatabaseQuerying {
        object: String,
        #[source]
        source: rusqlite::Error,
    },
}
