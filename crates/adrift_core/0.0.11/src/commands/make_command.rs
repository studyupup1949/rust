use crate::commands::Command;

use convert_case::{Case, Casing};
use std::fs::File;
use std::io::prelude::*;
use std::{collections::HashMap, env};

pub struct MakeCommand;

#[async_trait::async_trait]
impl Command for MakeCommand {
    fn name(&self) -> &'static str {
        "make:command"
    }

    fn description(&self) -> &'static str {
        "Create command"
    }

    fn args(&self) -> Vec<clap::Arg> {
        vec![clap::arg!([name])]
    }

    fn require_rebuild(&self) -> bool {
        true
    }

    async fn handle(&self, args: HashMap<String, String>) -> anyhow::Result<()> {
        let name = args.get("name").expect("Name argument is required!");
        
        let file_name = name.to_case(Case::Snake);
        let command_name = file_name.replace('_', ":");
        let struct_name = name.to_case(Case::UpperCamel);

        let file_path = format!(
            "{}/commands/{}.rs",
            env::var("CARGO_MANIFEST_DIR").unwrap(),
            file_name
        );

        let file_dir = format!(
            "{}/commands",
            env::var("CARGO_MANIFEST_DIR").unwrap(),
        );

        println!("Creating command {}..", struct_name);

        std::fs::create_dir_all(file_dir).unwrap();

        let mut file = File::create(file_path).expect("Error encountered while creating file!");

        let text = format!(r#"use std::collections::HashMap;

use adrift::commands::Command;
use adrift::anyhow;

pub struct {struct_name};

#[adrift::async_trait]
impl Command for {struct_name} {{
    fn name(&self) -> &'static str {{
        "{command_name}"
    }}

    async fn handle(&self, _args: HashMap<String, String>) -> anyhow::Result<()> {{
        Ok(())
    }}
}}
        "#);

        file.write_all(text.as_bytes())?;

        Ok(())
    }
}
