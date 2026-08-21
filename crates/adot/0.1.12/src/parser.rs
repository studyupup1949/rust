use std::collections::HashMap;
use std::path::PathBuf;

use yaml_rust2::{Yaml, YamlLoader};

use crate::config::{Config, Dotfile, DotfileType, Profile, Variable};

pub fn parse(content: &str) -> Result<Config, String> {
    let docs = YamlLoader::load_from_str(content).map_err(|e| format!("yaml parse error: {e}"))?;
    if docs.is_empty() {
        return Err("empty yaml document".to_string());
    }

    let doc = &docs[0];
    let mut config = Config::default();

    parse_config_section(doc, &mut config);
    parse_dotfiles(doc, &mut config)?;
    parse_profiles(doc, &mut config);
    config.variables = parse_variables(&doc["variables"]);
    config.dynvariables = parse_string_map(&doc["dynvariables"]);

    Ok(config)
}

fn parse_config_section(doc: &Yaml, config: &mut Config) {
    let section = &doc["config"];
    if section.is_badvalue() {
        return;
    }

    if let Some(dotpath) = section["dotpath"].as_str() {
        config.dotpath = PathBuf::from(dotpath);
    }
}

fn parse_dotfiles(doc: &Yaml, config: &mut Config) -> Result<(), String> {
    let section = &doc["dotfiles"];
    let hash = match section.as_hash() {
        Some(h) => h,
        None => return Ok(()),
    };

    for (key, value) in hash.iter() {
        // yaml-rust2 always parses mapping keys as strings
        let name = key.as_str().expect("yaml key must be a string");

        let dst = value["dst"]
            .as_str()
            .ok_or_else(|| format!("dotfile '{name}': missing 'dst'"))?;

        let src = value["src"]
            .as_str()
            .ok_or_else(|| format!("dotfile '{name}': missing 'src'"))?;

        let dtype = match value["type"].as_str() {
            Some("link") => DotfileType::Link,
            Some("copy") => DotfileType::Copy,
            Some("template") => DotfileType::Template,
            Some("link_children") => DotfileType::LinkChildren,
            Some(other) => return Err(format!("dotfile '{name}': unknown type '{other}'")),
            None => return Err(format!("dotfile '{name}': missing 'type'")),
        };

        config.dotfiles.insert(
            name.to_string(),
            Dotfile {
                dst: expand_tilde(dst),
                src: expand_tilde(src),
                dtype,
            },
        );
    }

    Ok(())
}

fn parse_profiles(doc: &Yaml, config: &mut Config) {
    let hash = match doc["profiles"].as_hash() {
        Some(h) => h,
        None => return,
    };

    for (key, value) in hash.iter() {
        let name = key.as_str().expect("yaml key must be a string");

        config.profiles.insert(
            name.to_string(),
            Profile {
                dotfiles: parse_string_list(&value["dotfiles"]),
                include: parse_string_list(&value["include"]),
                variables: parse_variables(&value["variables"]),
                dynvariables: parse_string_map(&value["dynvariables"]),
            },
        );
    }
}

fn parse_variables(yaml: &Yaml) -> HashMap<String, Variable> {
    let mut vars = HashMap::new();
    let hash = match yaml.as_hash() {
        Some(h) => h,
        None => return vars,
    };

    for (key, value) in hash.iter() {
        // yaml-rust2 always parses mapping keys as strings
        let name = key.as_str().expect("yaml key must be a string");
        vars.insert(name.to_string(), parse_variable_value(value));
    }

    vars
}

fn parse_variable_value(yaml: &Yaml) -> Variable {
    if let Some(hash) = yaml.as_hash() {
        let mut nested = HashMap::new();
        for (k, v) in hash.iter() {
            let key = k.as_str().expect("yaml key must be a string");
            nested.insert(key.to_string(), parse_variable_value(v));
        }
        return Variable::Nested(nested);
    }

    let s = yaml_to_string(yaml);
    Variable::Value(s)
}

fn parse_string_list(yaml: &Yaml) -> Vec<String> {
    let arr = match yaml.as_vec() {
        Some(a) => a,
        None => return Vec::new(),
    };

    arr.iter()
        .filter_map(|item| item.as_str().map(String::from))
        .collect()
}

fn parse_string_map(yaml: &Yaml) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let hash = match yaml.as_hash() {
        Some(h) => h,
        None => return map,
    };

    for (key, value) in hash.iter() {
        let k = key.as_str().expect("yaml key must be a string");
        map.insert(k.to_string(), yaml_to_string(value));
    }

    map
}

/// Convert a Yaml scalar to String, handling integers, bools, etc.
fn yaml_to_string(yaml: &Yaml) -> String {
    match yaml {
        Yaml::String(s) => s.clone(),
        Yaml::Integer(i) => i.to_string(),
        Yaml::Real(r) => r.clone(),
        Yaml::Boolean(b) => b.to_string(),
        _ => String::new(),
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if !path.starts_with('~') {
        return PathBuf::from(path);
    }

    match std::env::var("HOME") {
        Ok(home) => PathBuf::from(path.replacen('~', &home, 1)),
        Err(_) => PathBuf::from(path),
    }
}
