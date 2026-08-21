use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const HOME_ROW_KEYS: &[char] = &['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'];

pub struct App {
    pub start_dir: PathBuf,
    pub current_dir: PathBuf,
    pub user_input: String,
    pub output: OutputType,
    pub subdirectories: BTreeMap<String, PathBuf>,
    pub show_hidden: bool,
}

impl App {
    pub fn new(current_dir: PathBuf, show_hidden: bool) -> App {
        let mut new_app = App {
            start_dir: current_dir.clone(),
            current_dir,
            user_input: String::new(),
            output: OutputType::Current,
            subdirectories: BTreeMap::new(),
            show_hidden,
        };
        new_app.update_subdirectories();
        new_app
    }

    pub fn set_current_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
        self.update_subdirectories();
    }

    pub fn update_subdirectories(&mut self) {
        // remove all entries from subdirectories
        self.subdirectories.clear();
        let keys = generate_two_char_keys(HOME_ROW_KEYS);
        // find all the subdirectories within the current_dir add them to subdirectories, if
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            let mut key_index = 0;
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        if !self.show_hidden {
                            if let Some(name) = path.file_name() {
                                if name.to_str().map_or(false, |s| s.starts_with('.')) {
                                    continue;
                                }
                            }
                        }
                        if key_index < keys.len() {
                            self.subdirectories.insert(keys[key_index].clone(), path);
                            key_index += 1;
                        }
                    }
                }
            }
        }
    }

    pub fn delete_input_letter(&mut self) {
        self.user_input.pop();
    }

    pub fn input_letter(&mut self, letter: char) {
        self.user_input.push(letter);
        if self.user_input.len() == 2 {
            if let Some(dir) = self.subdirectories.get(&self.user_input) {
                self.set_current_dir(dir.clone());
            }
            self.delete_input_letter();
            self.delete_input_letter();
        }
    }
}

fn generate_two_char_keys(chars: &[char]) -> Vec<String> {
    let mut keys = Vec::new();
    for &c1 in chars {
        for &c2 in chars {
            if c1 != c2 {
                keys.push(format!("{}{}", c1, c2));
            }
        }
    }
    keys
}

pub enum OutputType {
    Start,
    Current,
}
