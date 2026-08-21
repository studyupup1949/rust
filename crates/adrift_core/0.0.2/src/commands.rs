pub mod inspire;
pub mod make_command;
pub mod serve;

use std::collections::HashMap;
use std::fmt::Debug;

use async_trait::async_trait;

pub type Arg = clap::Arg;
pub use clap::arg;

#[async_trait]
pub trait Command {
    fn name(&self) -> &'static str;

    async fn handle(
        &self,
        args: HashMap<String, String>,
    ) -> anyhow::Result<()>;

    fn description(&self) -> &'static str {
        ""
    }

    fn require_rebuild(&self) -> bool {
        false
    }

    fn require_full_rebuild(&self) -> bool {
        false
    }

    fn args(&self) -> Vec<Arg> {
        vec![]
    }
}

impl Debug for dyn Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug)]
pub struct Commands {
    pub items: Vec<Box<dyn Command>>,
}
