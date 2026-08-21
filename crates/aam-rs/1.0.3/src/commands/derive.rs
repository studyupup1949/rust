use crate::aaml::AAML;
use crate::commands::Command;
use crate::error::AamlError;

pub struct DeriveCommand;

impl Command for DeriveCommand {
    fn name(&self) -> &str { "derive" }

    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError> {
        let raw_path = args.trim();
        if raw_path.is_empty() {
            return Err(AamlError::DirectiveError("derive".into(), "Missing file path".into()));
        }

        let path = AAML::unwrap_quotes(raw_path);
        let mut base = AAML::load(path)?;

        for (schema_name, schema) in base.get_schemas_mut().drain() {
            aaml.get_schemas_mut().entry(schema_name).or_insert(schema);
        }

        for (k, v) in base.get_map_mut().drain() {
            aaml.get_map_mut().entry(k).or_insert(v);
        }

        Ok(())
    }
}


