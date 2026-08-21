use std::collections::HashMap;

use crate::commands::{Arg, Command};

pub struct Clean;

#[async_trait::async_trait]
impl Command for Clean {
    fn name(&self) -> &'static str {
        "clean"
    }

    fn description(&self) -> &'static str {
        "Clean cached macros"
    }

    fn args(&self) -> Vec<Arg> {
        vec![]
    }

    fn require_rebuild(&self) -> bool {
        true
    }

    async fn handle(&self, _args: HashMap<String, String>) -> anyhow::Result<()> {
        println!("Clean cached macros");

        Ok(())
    }
}
