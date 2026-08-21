// Search for an abbreviation!
use colored::Colorize;
use serde_json::Value;
use std::collections::HashMap;

pub fn search(abbr: &str, abbreviations: &HashMap<String, Value>) {
    let abbr_lower = abbr.to_lowercase();

    if let Some(value) = abbreviations.get(&abbr_lower) {
        match value {
            Value::String(meaning) => {
                println!("{}: {}", "Meaning".bold(), meaning);
            }

            Value::Array(means) => {
                // print all the meanings
                for (i, meaning) in means.iter().enumerate() {
                    if let Some(meaning_str) = meaning.as_str() {
                        println!("{} {}: {}", "Meaning".bold(), i + 1, meaning_str);
                    }
                }
            }

            _ => {
                println!("{}: Unexpected data type for the abbreviation!\nCheck the value of this abbreviation in the abbreviation.json and make sure it is a String or Array.", "Error".red().bold());
            }
        }
    } else {
        println!("{}: The abbreviation you entered does not exist in the abbreviation.json file!", "Error".red().bold());
    }
}