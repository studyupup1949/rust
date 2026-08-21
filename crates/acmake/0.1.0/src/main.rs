
use clap::Parser;
use cli::{AcmakeArgs, Action};

use crate::generator::generate_project;
use crate::util::{create_dir, get_current_path};

mod cli;
mod generator;
mod model;
mod util;

fn main() -> Result<(), String> {
    let args = AcmakeArgs::parse();

    match args.action {
        Action::New {
            name,
            language,
            folder,
        } => {
            println!("{:?}, {:?}, {:?}", name, language, folder);

            let folder_name = match folder {
                Some(folder) => folder,
                None => name.clone(),
            };
            let current_dir = get_current_path().map_err(|e| e.to_string())?;
            let dir = create_dir(&current_dir, &folder_name).map_err(|e| e.to_string())?;

            generate_project(dir, name, language).map_err(|e| e.to_string())?;

            Ok(())
        }
        Action::Init { name: _, language: _ } => todo!(),
    }
}
