use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadingError {
    #[error("No solution library found for year {year}")]
    LibraryNotFound {
        year: i32,
        search_path: PathBuf,
        #[source]
        source: Option<std::io::Error>,
    },

    #[error("Multiple solution libraries found: {}", .paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "))]
    AmbiguousLibraries { paths: Vec<PathBuf> },

    #[error("Failed to load solution library: {reason}")]
    LibraryLoadFailed {
        reason: String,
        #[source]
        source: Option<libloading::Error>,
    },

    #[error("No solutions found in library for year {year}")]
    NoSolutions { year: i32 },

    #[error("Invalid solution library: {reason}")]
    InvalidLibrary { reason: String },
}

impl LoadingError {
    pub fn library_load_failed(e: libloading::Error) -> Self {
        LoadingError::LibraryLoadFailed {
            reason: e.to_string(),
            source: Some(e),
        }
    }

    pub fn invalid_library(reason: impl Into<String>) -> Self {
        LoadingError::InvalidLibrary {
            reason: reason.into(),
        }
    }

    pub fn no_solutions(year: i32) -> Self {
        LoadingError::NoSolutions { year }
    }

    pub fn library_not_found(
        year: i32,
        search_path: PathBuf,
        source: Option<std::io::Error>,
    ) -> Self {
        LoadingError::LibraryNotFound {
            year,
            search_path,
            source,
        }
    }

    pub fn ambiguous_libraries(paths: Vec<PathBuf>) -> Self {
        LoadingError::AmbiguousLibraries { paths }
    }
}
