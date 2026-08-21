use std::collections::HashMap;

use crate::commands::{Arg, Command};

pub struct Inspire;

#[async_trait::async_trait]
impl Command for Inspire {
    fn name(&self) -> &'static str {
        "inspire"
    }

    fn description(&self) -> &'static str {
        "Display an inspiring quote"
    }

    fn args(&self) -> Vec<Arg> {
        vec![]
    }

    async fn handle(&self, _args: HashMap<String, String>) -> anyhow::Result<()> {
        println!("Display an inspiring quote");

        Ok(())
    }
}
