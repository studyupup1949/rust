use crate::sql::sqlite;
use std::path::Path;

extern crate custom_error;
use custom_error::custom_error;
extern crate rusqlite;

custom_error! {pub NewRepoError
    FolderNotFound = "Folder not found!",
    SQLError{source: rusqlite::Error} = "SQL Error"
}

/// Generates a new repository.
/// * `name` : Repository name (used for the Build command and displayed on the GUI)
/// * `path` : Path on your machine to the mods folder (can be relative or absolute)
/// * `url` : URL to the mods folder. (A3MO generates an json file inside the folder)
/// * `delta_patch` : Only transfers updated file chunks to the client, instead of updating the complete file
pub fn new(name: &str, path: &str, url: &str) -> Result<(), NewRepoError> {
    if !Path::exists(path.as_ref()) {
        return Err(NewRepoError::FolderNotFound);
    };
    let mut conn = sqlite::get_conn()?;
    sqlite::insert_repository(name, path, url, &mut conn)?;
    Ok(())
}
