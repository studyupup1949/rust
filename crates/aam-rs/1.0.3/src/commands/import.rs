use crate::aaml::AAML;
use crate::commands::Command;
use crate::error::AamlError;

pub struct ImportCommand;

impl Command for ImportCommand {
    fn name(&self) -> &str {
        "import"
    }

    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError> {
        let raw_path = args.trim();
        if raw_path.is_empty() {
            return Err(AamlError::ParseError {
                line: 0,
                content: args.to_string(),
                details: "Import path cannot be empty".to_string(),
            });
        }

        let path = AAML::unwrap_quotes(raw_path);
        aaml.merge_file(path)
    }
}
