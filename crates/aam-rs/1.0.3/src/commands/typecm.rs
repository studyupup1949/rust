use crate::commands::Command;
use crate::error::AamlError;
use crate::types::{resolve_builtin, Type};
use crate::types::primitive_type::PrimitiveType;

pub enum TypeDefinition {
    Primitive(String),
    Alias(String),
    Builtin(String),
}

impl Type for TypeDefinition {
    fn from_name(_name: &str) -> Result<Self, AamlError>
    where
        Self: Sized,
    {
        Err(AamlError::NotFound("TypeDefinition::from_name not supported".to_string()))
    }

    fn base_type(&self) -> PrimitiveType {
        match self {
            TypeDefinition::Builtin(path) => {
                resolve_builtin(path).map(|t| t.base_type()).unwrap_or(PrimitiveType::String)
            }
            TypeDefinition::Primitive(name) => {
                PrimitiveType::from_name(name).unwrap_or(PrimitiveType::String).base_type()
            }
            TypeDefinition::Alias(_) => PrimitiveType::String,
        }
    }

    fn validate(&self, value: &str) -> Result<(), AamlError> {
        match self {
            TypeDefinition::Builtin(path) => {
                resolve_builtin(path)?.validate(value)
            }
            TypeDefinition::Primitive(name) => {
                PrimitiveType::from_name(name)?.validate(value)
            }
            TypeDefinition::Alias(_) => Ok(()),
        }
    }
}

pub struct TypeCommand;

impl Command for TypeCommand {
    fn name(&self) -> &str {
        "type"
    }

    fn execute(&self, aaml: &mut crate::aaml::AAML, args: &str) -> Result<(), AamlError> {
        let (name, definition) = args.split_once('=').ok_or_else(|| AamlError::ParseError {
            line: 0,
            content: args.to_string(),
            details: "Type definition must be in the format 'name = definition'".to_string(),
        })?;

        let name = name.trim();
        let definition = definition.trim();

        if name.is_empty() {
            return Err(AamlError::ParseError {
                line: 0,
                content: args.to_string(),
                details: "Type name cannot be empty".to_string(),
            });
        }
        if definition.is_empty() {
            return Err(AamlError::ParseError {
                line: 0,
                content: args.to_string(),
                details: "Type definition cannot be empty".to_string(),
            });
        }

        let type_def = if definition.contains("::") {
            TypeDefinition::Builtin(definition.to_string())
        } else {
            TypeDefinition::Primitive(definition.to_string())
        };

        aaml.register_type(name.to_string(), type_def);

        Ok(())
    }
}