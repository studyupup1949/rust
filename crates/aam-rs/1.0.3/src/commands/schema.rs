use std::collections::HashMap;
use crate::aaml::AAML;
use crate::commands::Command;
use crate::error::AamlError;

#[derive(Clone, Debug)]
pub struct SchemaDef {
    pub fields: HashMap<String, String>,
}

pub struct SchemaCommand;

impl SchemaCommand {
    fn parse(args: &str) -> Result<(String, SchemaDef), AamlError> {
        let args = args.trim();
        let (name_part, body_part) = args.split_once('{')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), "Expected '{'".into()))?;

        let name = name_part.trim();
        if name.is_empty() {
            return Err(AamlError::DirectiveError("schema".into(), "Schema name is empty".into()));
        }

        let body = body_part.rsplit_once('}')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), "Expected '}'".into()))?
            .0;

        let mut fields = HashMap::new();
        for item in body.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let (field, ty) = item.split_once(':')
                .ok_or_else(|| AamlError::DirectiveError("schema".into(), format!("Bad field: {item}")))?;
            let field = field.trim();
            let ty = ty.trim();
            if field.is_empty() || ty.is_empty() {
                return Err(AamlError::DirectiveError("schema".into(), format!("Bad field: {item}")));
            }
            fields.insert(field.to_string(), ty.to_string());
        }

        Ok((name.to_string(), SchemaDef { fields }))
    }
}

impl Command for SchemaCommand {
    fn name(&self) -> &str { "schema" }

    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError> {
        let (name, schema) = Self::parse(args)?;
        aaml.get_schemas_mut().insert(name, schema);

        Ok(())
    }
}
