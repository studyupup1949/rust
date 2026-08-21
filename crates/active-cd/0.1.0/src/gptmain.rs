use anyhow::Result;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use crossterm::{
    execute,
    event::{read, Event, KeyCode, KeyEvent},
    terminal::{self, Clear, ClearType},
};

const HOME_ROW_KEYS: &[char] = &['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'];

fn main() -> Result<()> {
    let mut current_dir = env::current_dir().expect("Failed to get current directory");

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    loop {
        // Clear and display directories
        execute!(stdout, Clear(ClearType::All))?;
        let subdirs = list_subdirectories(&current_dir);
        display_directories(&subdirs);

        // Show current directory at the top
        println!("Current Directory: {}", current_dir.display());

        // Wait for user input
        if let Event::Key(KeyEvent { code, .. }) = read()? {
            match code {
                KeyCode::Char('q') => {
                    // Exit the program and navigate to the last visited directory
                    terminal::disable_raw_mode()?;
                    env::set_current_dir(&current_dir)
                        .expect("Failed to set terminal directory");
                    break;
                }
                KeyCode::Char('u') => {
                    // Navigate up a directory
                    if let Some(parent) = current_dir.parent() {
                        current_dir = parent.to_path_buf();
                    }
                }
                KeyCode::Char('~') => {
                    // Navigate to the home directory
                    if let Some(home_dir) = dirs::home_dir() {
                        current_dir = home_dir;
                    } else {
                        println!("Unable to find the home directory.");
                    }
                }
                KeyCode::Char('/') => {
                    // Navigate to the root directory
                    current_dir = PathBuf::from("/");
                }
                KeyCode::Char(c) => {
                    // Navigate into a subdirectory based on the key combination
                    if let Some(subdir) = subdirs.get(&c.to_string()) {
                        current_dir.push(subdir);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn list_subdirectories(path: &Path) -> std::collections::HashMap<String, String> {
    let mut key_iterator = HOME_ROW_KEYS.iter();
    let mut subdirs = std::collections::HashMap::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    if let Some(&key) = key_iterator.next() {
                        subdirs.insert(key.to_string(), entry.file_name().to_string_lossy().to_string());
                    } else {
                        break; // Stop if we run out of keys
                    }
                }
            }
        }
    }
    subdirs
}

fn display_directories(subdirs: &std::collections::HashMap<String, String>) {
    for (key, dir) in subdirs {
        println!("{} -> {}", key, dir);
    }
}
