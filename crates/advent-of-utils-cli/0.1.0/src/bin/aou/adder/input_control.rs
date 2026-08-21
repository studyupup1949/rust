use std::env;

use advent_of_utils_cli::{error::AocError, types::AocDatabase, Parts};
use std::io::{Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;

/// Returns true if the given editor is available on the system
fn check_installed(editor: &str) -> bool {
    Command::new("which")
        .arg(editor)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Opens the test input in the editor set with $EDITOR or else with Vi(m)
pub fn edit_input(
    year: i32,
    day: u8,
    db: &AocDatabase,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(
        db.get_input(year, day, true)
            .unwrap_or("".to_string())
            .as_bytes(),
    )?;
    temp_file.flush()?;

    let editor: String = match env::var("EDITOR") {
        Ok(editor) if check_installed(&editor) => editor,
        Ok(_) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "$EDITOR set but editor not installed",
            )))
        }
        Err(env::VarError::NotPresent) => {
            if check_installed("vim") {
                "vim".to_string()
            } else {
                "vi".to_string()
            }
        }
        Err(error) => {
            return Err(Box::new(error));
        }
    };

    let status = Command::new(editor).arg(temp_file.path()).status()?;

    if !status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Editor returned non-zero exit status",
        )));
    }

    // Read the modified content
    let mut new_content = String::new();
    temp_file.reopen()?.read_to_string(&mut new_content)?;

    // Update the database
    db.set_input(year, day, true, new_content)?;
    Ok(())
}

// fn get_result(year: i32, day: u8, part: Parts, db: &AocDatabase) -> Result<(), AocError> {
//     println!("Expected Test Result for {part}. Leave empty for keeping the current set Result. Must be set before you are able to run the test for this part:\n");
//
//     Ok(())
// }
//
// /// Get test results from the command line
// pub fn get_results(year: i32, day: u8, db: &AocDatabase) -> Result<(), AocError> {
//     get_result(year, day, Parts::Part1, db)?;
//     get_result(year, day, Parts::Part2, db)?;
//     Ok(())
// }
