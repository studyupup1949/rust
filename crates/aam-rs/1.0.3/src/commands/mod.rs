use crate::aaml::AAML;
use crate::error::AamlError;

pub mod import;
pub mod schema;
pub mod typecm;
pub mod derive;

pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError>;
}