// SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
//
// SPDX-License-Identifier: ISC

use std::io;
use std::{fs, path::PathBuf};

use indexmap::map::IndexMap;
use mailparse::MailHeaderMap;
use miette::IntoDiagnostic;
use serde::{de, ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Deserialize, Serialize)]
pub struct AddressBook {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    path: PathBuf,
    #[serde(serialize_with = "AddressBook::serialize_entries")]
    #[serde(deserialize_with = "AddressBook::deserialize_entries")]
    #[serde(rename = "address")]
    entries: IndexMap<String, String>,
}

impl AddressBook {
    fn serialize_entries<S>(entries: &IndexMap<String, String>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = s.serialize_seq(Some(entries.len()))?;

        for (k, v) in entries {
            ser.serialize_element(&IndexMap::from([
                ("name", k.as_str()),
                ("email", v.as_str()),
            ]))?;
        }
        ser.end()
    }

    fn deserialize_entries<'de, D>(deserializer: D) -> Result<IndexMap<String, String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let res: Result<Vec<IndexMap<String, String>>, D::Error> = Vec::deserialize(deserializer);
        let mut indmap: IndexMap<String, String> = IndexMap::new();
        match res {
            Ok(n) => {
                for e in n.iter() {
                    let Some(name) = e.get("name") else {
                        return Err(de::Error::missing_field("name"));
                    };
                    let Some(email) = e.get("email") else {
                        return Err(de::Error::missing_field("email"));
                    };
                    indmap.insert(name.to_string(), email.to_string());
                }

                Ok(indmap)
            }
            Err(e) => Err(de::Error::custom(e.to_string())),
        }
    }

    fn write_to_file(&self) -> miette::Result<()> {
        if self.entries.is_empty() {
            Ok(fs::write(&self.path, "").into_diagnostic()?)
        } else {
            Ok(
                fs::write(&self.path, toml::to_string(&self).into_diagnostic()?)
                    .into_diagnostic()?,
            )
        }
    }

    pub fn new(path: Option<PathBuf>) -> miette::Result<Self> {
        let path = match path.or_else(|| dirs::data_dir().map(|p| p.join("aba.toml"))) {
            Some(p) => p,
            None => miette::bail!("Could not set address book path based on XDG_DATA_DIR"),
        };

        match fs::read_to_string(&path) {
            Ok(s) => {
                let mut addbook: Self = toml::from_str(&s).into_diagnostic()?;
                addbook.path = path;
                Ok(addbook)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .into_diagnostic()?;
                Ok(AddressBook {
                    entries: IndexMap::new(),
                    path,
                })
            }
            Err(e) => Err(e).into_diagnostic(),
        }
    }

    pub fn add(&mut self, name: &str, email: &str, force: bool) -> miette::Result<()> {
        if !self.entries.is_empty() && self.entries.contains_key(name) && !force {
            miette::bail!("Cannot add \"{}\"; it already exists", name)
        }

        self.entries.insert(name.to_string(), email.to_string());

        self.write_to_file()?;

        println!("Add \"{}\" with email \"{}\"", name, email);

        Ok(())
    }

    pub fn list(&self, fuzzy: &str, number: usize) -> miette::Result<()> {
        let names: Vec<&str> = self.entries.keys().map(String::as_ref).collect();
        let matches = rust_fuzzy_search::fuzzy_search_best_n(fuzzy, &names, number);
        for (m, _) in matches {
            println!("\"{}\" <{}>", m, self.entries.get(m).unwrap());
        }
        Ok(())
    }

    pub fn del(&mut self, match_str: &str, is_regex: bool) -> miette::Result<()> {
        // TODO: Find a way to print removed entries
        if is_regex {
            let cmpregex = regex::Regex::new(match_str).into_diagnostic()?;
            self.entries.retain(|k, _| !cmpregex.is_match(k));
        } else {
            self.entries.remove(match_str);
        }

        self.write_to_file()?;

        Ok(())
    }

    pub fn parse(
        &mut self,
        filename: &Option<PathBuf>,
        force: bool,
        from: bool,
        cc: bool,
        to: bool,
        all: bool,
    ) -> miette::Result<()> {
        let raw_mail = match filename {
            Some(p) if p.to_str() == Some("-") => {
                io::read_to_string(io::stdin()).into_diagnostic()?
            }
            Some(p) => fs::read_to_string(p).into_diagnostic()?,
            None => io::read_to_string(io::stdin()).into_diagnostic()?,
        };

        let parsed_mail = mailparse::parse_mail(raw_mail.as_bytes()).into_diagnostic()?;
        let regexes: (regex::Regex, regex::Regex) = (
            regex::Regex::new(r"(.*) <(.*)>").into_diagnostic()?,
            regex::Regex::new(r"(.*)(@.*)").into_diagnostic()?,
        );
        let possible_headers: Vec<(bool, &str)> = vec![
            // From as the default value
            (from || all || (!to && !cc), "From"),
            (to || all, "To"),
            (cc || all, "Cc"),
        ];
        let mut new_entries: Vec<(String, String)> = Vec::new();

        for (use_header, header) in possible_headers {
            if !use_header {
                continue;
            }

            let values = parsed_mail.headers.get_all_headers(header);

            for v in values {
                let full_address = v.get_value_utf8().into_diagnostic()?;
                match regexes.0.captures(&full_address).map(|c| c.extract()) {
                    Some((_, [name, email])) => {
                        if !force && self.entries.contains_key(name) {
                            miette::bail!("Cannot add \"{}\"; it already exists", name)
                        }

                        new_entries.push((name.to_string(), email.to_string()));
                        self.entries.insert(name.to_string(), email.to_string());
                    }
                    None => match regexes.1.captures(&full_address).map(|c| c.extract()) {
                        Some((email, [user, _])) => {
                            if !force && self.entries.contains_key(user) {
                                miette::bail!("Cannot add \"{}\"; it already exists", user)
                            }

                            new_entries.push((user.to_string(), email.to_string()));
                            self.entries.insert(user.to_string(), email.to_string());
                        }
                        None => miette::bail!("Could not match address -- `{}'", full_address),
                    },
                }
            }
        }

        self.write_to_file()?;

        for e in new_entries {
            println!("Added: name \"{}\", email \"{}\"", e.0, e.1);
        }

        Ok(())
    }
}
