use advent_of_utils::Solution;
use advent_of_utils_cli::error::{AocError, LoadingError};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::RunConfig;

#[repr(C)]
struct RawSolutions {
    solutions: *mut HashMap<u8, Box<dyn Solution>>,
}

pub(super) struct SolutionLibrary {
    lib: Arc<Library>,
    year: i32,
}

impl SolutionLibrary {
    pub fn load(config: &RunConfig) -> Result<Self, AocError> {
        let path = find_library(config)?;

        unsafe {
            let lib = Library::new(&path).map_err(|e| {
                AocError::Loading(LoadingError::LibraryLoadFailed {
                    reason: e.to_string(),
                    source: Some(e),
                })
            })?;

            lib.get::<Symbol<extern "C" fn() -> *mut RawSolutions>>(b"create_solutions")
                .map_err(|e| {
                    AocError::Loading(LoadingError::InvalidLibrary {
                        reason: format!("Missing required symbol 'create_solutions': {}", e),
                    })
                })?;

            Ok(Self {
                lib: Arc::new(lib),
                year: config.year,
            })
        }
    }

    pub fn get_solutions(&self) -> Result<HashMap<u8, Box<dyn Solution>>, AocError> {
        unsafe {
            let create_solutions = self
                .lib
                .get::<Symbol<extern "C" fn() -> *mut RawSolutions>>(b"create_solutions")
                .map_err(|e| {
                    AocError::Loading(LoadingError::InvalidLibrary {
                        reason: format!("Failed to get 'create_solutions' symbol: {}", e),
                    })
                })?;

            let container = create_solutions();
            if container.is_null() || (*container).solutions.is_null() {
                return Err(AocError::Loading(LoadingError::NoSolutions {
                    year: self.year,
                }));
            }

            let solutions = Box::from_raw((*container).solutions);
            Ok(*solutions)
        }
    }
}

impl Drop for SolutionLibrary {
    fn drop(&mut self) {
        unsafe {
            if let Ok(destroy) = self
                .lib
                .get::<Symbol<unsafe extern "C" fn(*mut RawSolutions)>>(b"destroy_solutions")
            {
                let container = Box::into_raw(Box::new(RawSolutions {
                    solutions: std::ptr::null_mut(),
                }));
                destroy(container);
            }
        }
    }
}

fn find_library(config: &RunConfig) -> Result<PathBuf, AocError> {
    let paths = config.loader_paths().map_err(|_| {
        AocError::Loading(LoadingError::LibraryNotFound {
            year: config.year,
            search_path: config.workspace_dir.clone(),
            source: None,
        })
    })?;

    if paths.is_empty() {
        return Err(AocError::Loading(LoadingError::LibraryNotFound {
            year: config.year,
            search_path: config.workspace_dir.clone(),
            source: None,
        }));
    }

    let mut valid_paths = Vec::new();
    for entry in paths {
        let path = entry.path();
        if unsafe { Library::new(&path).is_ok() } {
            valid_paths.push(path);
        }
    }

    match valid_paths.len() {
        0 => Err(AocError::Loading(LoadingError::NoSolutions {
            year: config.year,
        })),
        1 => Ok(valid_paths.remove(0)),
        _ => Err(AocError::Loading(LoadingError::AmbiguousLibraries {
            paths: valid_paths,
        })),
    }
}
