// Lists all the abbreviations in the list!

use colored::Colorize;
use serde_json::Value;
use std::collections::HashMap;

pub fn list(abbreviations: &HashMap<String, Value>) {
    if abbreviations.is_empty() {
        println!("{}", "No abbreviations available!".red().bold());
        return;
    }

    for (abbr, value) in abbreviations {
        print_abbreviation_meaning(abbr, value);
    }
}

fn print_abbreviation_meaning(abbr: &str, value: &Value) {
    match value {
        Value::String(meaning) => {
            println!("{}: {} - {}", abbr.bold(), "Meaning".bold(), meaning);
        }

        Value::Array(means) => {
            for (i, meaning) in means.iter().enumerate() {
                if let Some(meaning_str) = meaning.as_str() {
                    println!("{}: {} - {}", abbr.bold(), format!("Meaning {}", i + 1).bold(), meaning_str);
                }
            }
        }

        _ => {
            println!("{}: Unexpected data type for abbreviation `{}`!\nCheck the value of this abbreviation in the abbreviation.json and make sure it is a String or Array.", "Error".red().bold(), abbr);
        }
    }
}
