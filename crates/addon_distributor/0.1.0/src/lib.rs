//! # Addon Distributor
//!
//! `addon_distributor` is a simple, fast tool to create, update, and manage json files that serve
//! as update file for firefox extensions

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::collections::BTreeMap;

use dialoguer::{theme::{ColorfulTheme, CustomPromptCharacterTheme}, Select, Input, Confirmation};
use serde::{Serialize, Deserialize};
use serde_json;
use clap::{Arg, App};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Gecko {
    #[serde(skip_serializing_if = "Option::is_none")]
    strict_min_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strict_max_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    advisory_max_version: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Application {
    gecko: Gecko
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Update {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    update_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    update_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    update_info_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multiprocess_compatible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    applications: Option<Application>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Addon {
   #[serde(skip_serializing_if = "Option::is_none")]
   updates: Option<Vec<Update>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Addons {
    addons: BTreeMap<String, Addon>
}

pub mod distributor {
    use super::*;

    pub fn get_addon_names(addons: &Addons) -> Vec<String> {
        return addons.addons.keys().cloned().collect();
    }

    pub fn upsert_addon(addons: &mut Addons, name: String, addon: Addon) {
        addons.addons.insert(name, addon);
    }

    pub fn insert_update_in_addon(addons: &mut Addons, name: String, update: Update) {
        for (addon_name, mut addon) in addons.addons.iter_mut() {
            if addon_name == &name {
                if addon.updates.is_some() {
                    addon.updates.as_mut().unwrap().insert(0, update.clone());
                } else {
                    addon.updates = Some(vec![update.clone()]);
                }
            }
        }
    }

    pub fn read_updates_from_file<P: AsRef<Path>>(path: P) -> Result<Addons,Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let updates: Addons = serde_json::from_reader(reader)?;

        Ok(updates)
    }

    pub fn write_updates_to_file<P: AsRef<Path>>(path: P, updates: &Addons) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let result = serde_json::to_writer_pretty(writer, &updates)?;
        Ok(result)
    }

    pub fn addon_chooser(selections: &Vec<String>) -> Option<usize> {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your addon")
            .items(&selections[..])
            .interact_opt()
            .unwrap();
        return selection;
    }

    pub fn prompt_for(prompt: &str) -> String {
        let theme = CustomPromptCharacterTheme::new('>');
        let name: String = Input::with_theme(&theme)
            .with_prompt(prompt)
            .interact()
            .unwrap();
        return name;
    }

    pub fn addon_name() -> String {
        prompt_for("\nWhat is the UUID of the add-on you want to add?\n")
    }

    pub fn select_addon_name(addons: &Addons) -> String {
        let mut choices = get_addon_names(addons);
        choices.push(String::from("New addon"));
        let choice_for_new = choices.len() - 1;
        let choice = addon_chooser(&choices).unwrap();
        if choice == choice_for_new {
            return addon_name();
        } else {
            return choices.remove(choice);
        }
    }

    pub fn optional_prompt_for(prompt: &str) -> Option<String> {
        let theme = CustomPromptCharacterTheme::new('>');
        let input: String = Input::with_theme(&theme)
            .with_prompt(prompt)
            .allow_empty(true)
            .interact()
            .unwrap();
        if input.is_empty() {
            return None;
        }
        return Some(String::from(input));
    }

    pub fn create_update() -> Update {
        Update {
            version: prompt_for("\nVersion?\n"),
            applications: None,
            update_hash: None,
            multiprocess_compatible: None,
            update_link: optional_prompt_for("\nA link to the XPI file containing this version of the add-on. This must be an HTTPS URL\n"),
            update_info_url: optional_prompt_for("\nA link to an HTML file containing information about the update (optional)\n"),
        }
    }

    pub fn confirm(prompt: &str) -> bool {
        Confirmation::new()
            .with_text(prompt)
            .interact()
            .unwrap()
    }

    pub fn updater(file: &str) {
        let mut addons = read_updates_from_file(file).unwrap_or(Addons {addons: BTreeMap::new()});
        let addon_name = select_addon_name(&addons); // gets the name of the add-on. Could even be an add-on that does not exist.
        println!("\nChose {}. On to creating update\n", addon_name);
        let update = create_update(); // just creates an update object
        if addons.addons.contains_key(&addon_name) {
            insert_update_in_addon(&mut addons, addon_name, update);
        } else {
            let new_addon = Addon { updates: Some(vec![update])};
            upsert_addon(&mut addons, addon_name, new_addon);
        }
        if confirm("Do you want to write the update to file?") {
            write_updates_to_file(file, &addons).expect("Some error");
            println!("Wrote. Thanks! Good luck for your next update.")
        } else {
            println!("Discarded update. See you again!")
        }
    }

    pub fn cli() {
        let matches = App::new("Addon Distributor")
            .version("0.1.0")
            .author("Akshay S Dinesh <asdofindia@gmail.com>")
            .about("Helps you create, update, and maintain an add-on update json file required for self-distributing firefox addons")
            .arg(Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("The json file to maintain. Default is updates.json")
                .takes_value(true))
            .get_matches();
        let updates = matches.value_of("file").unwrap_or("updates.json");
        println!("\nADDON DISTRIBUTOR\n\nChosen updates file: {}\n\nIf you chose the wrong file Ctrl+C to exit the program and choose the right file with the -f flag\n", updates);
        updater(updates);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILE : &str = "tests/test.json";

    #[test]
    fn read_updates_from_file() {
        let filename = TEST_FILE;
        distributor::read_updates_from_file(filename).expect("Unable to open file");
    }

    #[test]
    fn read_and_write() {
        let readfile = TEST_FILE;
        let writefile = TEST_FILE;
        let read = distributor::read_updates_from_file(readfile).unwrap();
        let write = distributor::write_updates_to_file(writefile, &read).unwrap();
        assert_eq!(write, ())
    }

    #[test]
    fn get_addon_names() {
        let addons = distributor::read_updates_from_file(TEST_FILE).unwrap();
        let addon_names = distributor::get_addon_names(addons);
        assert_eq!(addon_names, ["{abcd1234-1abc-1234-12ab-abcdef123456}"]);
    }

    #[test]
    fn upsert_addon() {
        let mut addons = distributor::read_updates_from_file(TEST_FILE).unwrap();
        let new_addon = Addon {
            updates: None
        };

        distributor::upsert_addon(&mut addons, String::from("testing"), new_addon);
    }

    #[test]
    fn select_addon() {
        let addons = distributor::read_updates_from_file(TEST_FILE).unwrap();
        println!("Choosen addon is {}", distributor::select_addon(addons));

        println!("Created update\n{:?}", distributor::create_update());
    }
}
