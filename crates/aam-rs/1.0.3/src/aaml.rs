use crate::commands::{self, Command};
use crate::error::AamlError;
use crate::found_value::FoundValue;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::{Add, AddAssign};
use std::path::Path;
use std::sync::Arc;
use crate::commands::schema::SchemaDef;
use crate::types::Type;

#[cfg(feature = "perf-hash")]
type Hasher = ahash::RandomState;

#[cfg(not(feature = "perf-hash"))]
type Hasher = std::collections::hash_map::RandomState;

type AamlString = Box<str>;

pub struct AAML {
    map: HashMap<AamlString, AamlString, Hasher>,
    commands: HashMap<String, Arc<dyn Command>>,
    types: HashMap<String, Box<dyn Type>>,
    schemas: HashMap<String, SchemaDef>,
}

impl std::fmt::Debug for AAML {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AAML")
            .field("map", &self.map)
            .field("commands_count", &self.commands.len())
            .finish()
    }
}

impl AAML {
    pub fn new() -> AAML {
        let mut instance = AAML {
            map: HashMap::with_hasher(Hasher::new()),
            commands: HashMap::new(),
            types: HashMap::new(),
            schemas: HashMap::new(),
        };
        instance.register_default_commands();
        instance
    }

    pub fn with_capacity(capacity: usize) -> AAML {
        let mut instance = AAML {
            map: HashMap::with_capacity_and_hasher(capacity, Hasher::default()),
            commands: HashMap::new(),
            types: HashMap::new(),
            schemas: HashMap::new(),
        };
        instance.register_default_commands();
        instance
    }

    pub(crate) fn get_schemas_mut(&mut self) -> &mut HashMap<String, SchemaDef> {
        &mut self.schemas
    }

    pub fn get_schema(&self, name: &str) -> Option<&SchemaDef> {
        self.schemas.get(name)
    }

    pub(crate) fn get_map_mut(&mut self) -> &mut HashMap<AamlString, AamlString, Hasher> {
        &mut self.map
    }


    pub fn check_type(&self, type_name: &str, value: &str) -> Result<(), AamlError> {
        if let Some(type_def) = self.types.get(type_name) {
            type_def.validate(value)
        } else {
            Err(AamlError::NotFound(type_name.to_string()))
        }
    }

    pub fn register_command<C>(&mut self, command: C)
    where
        C: Command + 'static,
    {
        self.commands.insert(command.name().to_string(), Arc::new(command));
    }

    pub fn register_type<T>(&mut self, name: String, type_def: T)
    where
        T: Type + 'static,
    {
        self.types.insert(name, Box::new(type_def));
    }

    pub fn get_type(&self, name: &str) -> Option<&Box<dyn Type>> {
        self.types.get(name)
    }

    pub fn validate_value(&self, type_name: &str, value: &str) -> Result<(), AamlError> {
        if let Some(type_def) = self.types.get(type_name) {
            type_def.validate(value).map_err(|e| AamlError::InvalidType {
                type_name: type_name.to_string(),
                details: e.to_string(),
            })
        } else {
            Err(AamlError::NotFound(type_name.to_string()))
        }
    }

    pub fn merge_content(&mut self, content: &str) -> Result<(), AamlError> {
        let estimated_size = content.len() / 40;
        self.map.reserve(estimated_size);

        for (i, line) in content.lines().enumerate() {
             self.process_line(&line, i + 1)?;
        }
        Ok(())
    }

    pub fn merge_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(), AamlError> {
        let content = fs::read_to_string(file_path)?;
        self.merge_content(&content)
    }

    pub fn parse(content: &str) -> Result<Self, AamlError> {
        let mut aaml = AAML::new();
        aaml.merge_content(content)?;
        Ok(aaml)
    }

    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, AamlError> {
        let content = fs::read_to_string(file_path)?;
        Self::parse(&content)
    }

    pub fn find_obj(&self, key: &str) -> Option<FoundValue> {
        self.map.get(key)
            .map(|v| FoundValue::new(v))
            .or_else(|| self.find_key(key))
    }

    pub fn find_deep(&self, key: &str) -> Option<FoundValue> {
        let mut current_key = key;
        let mut last_found = None;
        let mut visited = HashSet::with_hasher(Hasher::default());

        while let Some(next_val) = self.map.get(current_key) {
            if !visited.insert(current_key) {
                break;
            }

            if visited.contains(&**next_val) {
                if last_found.is_none() {
                    last_found = Some(next_val);
                }
                break;
            }

            last_found = Some(next_val);
            current_key = next_val;
        }

        last_found.map(|v| FoundValue::new(v))
    }

    pub fn find_key(&self, value: &str) -> Option<FoundValue> {
        self.map.iter()
            .find_map(|(k, v)| {
                if &**v == value {
                    Some(FoundValue::new(k))
                } else {
                    None
                }
            })
    }

    pub fn unregister_type(&mut self, name: &str) {
        self.types.remove(name);
    }

    fn register_default_commands(&mut self) {
        self.register_command(commands::import::ImportCommand);
        self.register_command(commands::typecm::TypeCommand);
        self.register_command(commands::schema::SchemaCommand);
        self.register_command(commands::derive::DeriveCommand);
    }

    fn process_line(&mut self, raw_line: &str, line_num: usize) -> Result<(), AamlError> {
        let line = Self::strip_comment(raw_line).trim();

        if line.is_empty() {
            return Ok(());
        }

        if let Some(rest) = line.strip_prefix('@') {
            return self.process_directive(rest, line_num);
        }

        match Self::parse_assignment(line) {
            Ok((key, value)) => {
                self.map.insert(Box::from(key), Box::from(value));
                Ok(())
            }
            Err(details) => Err(AamlError::ParseError {
                line: line_num,
                content: line.to_string(),
                details: details.to_string(),
            }),
        }
    }

    fn process_directive(&mut self, content: &str, line_num: usize) -> Result<(), AamlError> {
        let mut parts = content.splitn(2, char::is_whitespace);
        let command_name = parts.next().unwrap_or("").trim();
        let args = parts.next().unwrap_or("");

        if command_name.is_empty() {
            return Err(AamlError::ParseError {
                line: line_num,
                content: content.to_string(),
                details: "Empty directive".to_string(),
            });
        }

        let command = self.commands.get(command_name).cloned();

        if let Some(cmd) = command {
            cmd.execute(self, args)
        } else {
            Err(AamlError::ParseError {
                line: line_num,
                content: content.to_string(),
                details: format!("Unknown directive: @{}", command_name),
            })
        }
    }

    fn strip_comment(line: &str) -> &str {
        let mut quote_state = None;

        for (idx, c) in line.char_indices() {
             match (quote_state, c) {
                (None, '#') => return &line[..idx],
                (None, '"' | '\'') => quote_state = Some(c),
                (Some(q), c) if c == q => quote_state = None,
                _ => {}
            }
        }
        line
    }

    fn parse_assignment(line: &'_ str) -> Result<(&'_ str, &'_ str), &'static str> {
        let (key, val) = line.split_once('=')
            .ok_or("Missing assignment operator '='")?;

        let key = key.trim();
        if key.is_empty() {
            return Err("Key cannot be empty");
        }

        let value = Self::unwrap_quotes(val);
        Ok((key, value))
    }

    pub fn unwrap_quotes(s: &str) -> &str {
        let s = s.trim();
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            return &s[1..s.len() - 1];
        }
        if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
            return &s[1..s.len() - 1];
        }
        s
    }
}

impl Add for AAML {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self.map.reserve(rhs.map.len());
        self.map.extend(rhs.map);
        self.types.extend(rhs.types);
        self
    }
}

impl AddAssign for AAML {
    fn add_assign(&mut self, rhs: Self) {
        self.map.reserve(rhs.map.len());
        self.map.extend(rhs.map);
        self.types.extend(rhs.types);
    }
}

impl Default for AAML {
    fn default() -> Self {
        Self::new()
    }
}