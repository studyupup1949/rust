use crate::search_gitignore::search::*;
use crate::utils::*;
use std::path::Path;
use std::process;

use inquire::{Confirm, Select};

mod generate_gitignore;
mod search_gitignore;
mod utils;

#[tokio::main]
async fn main() {
    let lang_options: Vec<&str> = vec![
        "Rust", "Node", "Go", "Dart", "Java", "C", "C++", "python", "ruby", "lua", "Swift", "Ruby",
        "Haskell", "oCaml",
    ];

    let lang_options: Vec<String> = lang_options
        .iter()
        .map(|word| capitalize_first_letter(word))
        .collect();

    if Path::new(".gitignore").exists() {
        let answer = Confirm::new("There is a .gitignore already, Do you want to overwrite it?")
            .with_default(false)
            .prompt();

        match answer {
            Ok(true) => {}
            Ok(false) => {
                println!("I will not touch your already existing .gitignore");
                process::exit(0)
            }
            Err(e) => {
                if let Err(err) = validate_response(&e) {
                    eprint!("There was an error: more detailes: {err}");
                    process::exit(1)
                }
            }
        }
    }

    let answer = match Select::new("Choose the language to get .gitignore for", lang_options)
        .with_vim_mode(true)
        .prompt()
    {
        Ok(choice) => choice,
        Err(err) => {
            if let Err(err) = validate_response(&err) {
                eprint!("There was an error: more detailes: {err}");
                process::exit(1);
            }
            "".into()
        }
    };

    println!("Getting your gitignore ...");

    let search_result = search_for_gitignore(answer.to_string()).await;

    if let Err(error) = search_result {
        eprintln!("an error ocurred, details: {error}");
        process::exit(1);
    }

    println!(".gitignore generated in root dir");
    process::exit(0)
}
