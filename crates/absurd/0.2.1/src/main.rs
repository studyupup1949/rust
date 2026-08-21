#![allow(unreachable_code)]
mod error;

use std::{fmt::Debug, fs, path::{Path, PathBuf}};
use clap::{Args, Parser, Subcommand, ValueEnum};
use convert_case::{Case, Casing};

use crate::error::{Error, Result};
use absurd::append_after_marker;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Verb,
}

#[derive(Subcommand, Debug)]
enum Verb {
    /// Specify which type of component you want to create.
    #[command(subcommand)]
    Create(CreateCommand),
}

#[derive(Subcommand, Debug)]
enum CreateCommand {
    /// Controllers are a layer in between your models and stores.
    Controller(ComponentControllerArgs),

    /// Models represents any record from the store.
    Model(ComponentModelArgs),

    /// Stores handles the connection to a database.
    Store(ComponentStoreArgs),
}


#[derive(Args, Debug)]
struct ComponentControllerArgs {
    /// The name of the component. Path separators can be used to create sub folders.
    name: String,
}

#[derive(Args, Debug)]
struct ComponentModelArgs {
    /// The name of the component. Path separators can be used to create sub folders.
    name: String,

    /// The name of each field. Default type is serde::Value.
    #[arg(long, short, num_args(1..))]
    fields: Option<Vec<String>>,

    /// A table that can have varying fields.
    #[arg(long, conflicts_with("schemafull"), requires("fields"))]
    schemaless: bool,

    /// A table that has pre-defined table layout.
    #[arg(long, conflicts_with("schemaless"), requires("fields"))]
    schemafull: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SchemaType {
    Schemafull,
    Schemaless,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Engine {
    /// Local. File based database.
    File,

    /// Local. In-memory database which is cleared on shutdown.
    Mem,

    /// Local. RocksDb database.
    RocksDb,
}
impl Engine {
    fn pascal_case(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Mem => "Mem",
            Self::RocksDb => "RocksDb",
        }
    }

    fn snake_case(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Mem => "mem",
            Self::RocksDb => "rocks_db",
        }
    }
}

#[derive(Args, Debug)]
struct ComponentStoreArgs {
    /// The name of the component. Path separators can be used to create sub folders.
    name: String,

    /// One of the valid SurrealDB Storage Layers.
    #[arg(long, short, value_enum, default_value_t = Engine::Mem)]
    engine: Engine,
}


/// Component names should come in PascalCase.
#[derive(Clone)]
struct ComponentName(String);
impl ComponentName {
    fn pascal_case(&self) -> String {
        self.0.to_case(Case::Pascal).to_string()
    }

    fn snake_case(&self) -> String {
        self.0.to_case(Case::Snake).to_string()
    }
}
impl<'a> From<&'a str> for ComponentName {
    fn from(value: &'a str) -> Self {
        ComponentName(value.to_string())
    }
}
impl<'a> From<std::path::Component<'a>> for ComponentName {
    fn from(value: std::path::Component<'a>) -> Self {
        let value = value.as_os_str()
            .to_string_lossy()
            .to_string();
        Self(value)
    }
}
impl From<std::path::PathBuf> for ComponentName {
    fn from(value: std::path::PathBuf) -> Self {
        let last_folder = value.components().last().unwrap();
        let last_folder = Path::new(last_folder.as_os_str());
        let last_folder = last_folder.with_extension("");
        let last_folder = last_folder.to_string_lossy();
        let last_folder = last_folder.to_string();
        Self(last_folder)
    }
}
impl Debug for ComponentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ComponentName({}) | Snake: {} | Pascal: {}",
            self.0,
            self.snake_case(),
            self.pascal_case(),
        )
    }
}


fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Verb::Create(create_command) => {
            match create_command {
                CreateCommand::Controller(args) => handle_controller(args)?,
                CreateCommand::Model(args) => handle_model(args)?,
                CreateCommand::Store(args) => handle_store(args)?,
            };
        },
    };

    Ok(())
}

fn handle_controller(args: ComponentControllerArgs) -> Result<()> {
    let controller_name = ComponentName(args.name);
    let base_path = Path::new("src/controller");
    let controller_path = base_path.with_extension("rs");
    let controller_component_path = base_path.join(controller_name.snake_case()).with_extension("rs");

    if controller_component_path.try_exists()? {
        return Err(Error::ControllerAlreadyExists(controller_name.pascal_case()));
    }

    fs::create_dir_all(&controller_component_path.parent().unwrap())?;

    let controller_content = if controller_path.try_exists()? {
        let controller_content = fs::read_to_string(&controller_path)?;

        let mod_string = format!("mod {};", controller_name.snake_case());
        let controller_content = append_after_marker(controller_content, "pub mod", mod_string);

        controller_content
    } else {
        include_str!("../stubs/controller.txt")
            .replace("|ControllerNamePascal|", &controller_name.pascal_case())
            .replace("|controller_name_snake|", &controller_name.snake_case())
    };
    fs::write(controller_path, controller_content)?;

    let controller_component_content = include_str!("../stubs/components/controller.txt")
        .replace("|ControllerNamePascal|", &controller_name.pascal_case())
        .replace("|controller_name_snake|", &controller_name.snake_case());
    fs::write(controller_component_path, controller_component_content)?;

    Ok(())
}

fn handle_model(args: ComponentModelArgs) -> Result<()> {
    // Default to schemafull, so only match on an explicit schemaless, or if the
    // model_path contains a forward slash.
    let schema = if args.schemaless || (!args.schemafull && args.fields.is_none() && args.name.contains("/")) {
            SchemaType::Schemaless
        } else {
            SchemaType::Schemafull
        };

    // The full path, including `/`.
    let table = if args.name.contains("/") && schema == SchemaType::Schemaless && args.fields.is_none() {
        // Because Schemaless can appear in the CLI as:
        // $ create model Setting/Dark    # model is Setting.
        // However uf Schemaless is passed with args, then assume the user wants
        // nesting.
        ComponentName::from(Path::new(&args.name).components().nth_back(1).unwrap())
    } else {
        // And in Schemafull (or Schemaless with --fields) the CLI is:
        // $ create model Sub/Setting --schemafull    # model is Setting
        ComponentName::from(Path::new(&args.name).components().last().unwrap())
    };

    let base_path = Path::new("src/model");
    let model_path = args.name.to_lowercase();
    let component_path = base_path.join(&model_path).with_extension("rs");

    if component_path.exists() {
        return Err(Error::ModelAlreadyExists(table.pascal_case()));
    }

    // This is just setting up the `mod`s so that our nesting is respected.
    // Firstly we don't want to do this for the final folder, as that is the
    // model itself.
    let component_path_no_ext = component_path.with_extension("");

    let fields = if args.fields.is_some() {
            Ok(args.fields.clone().unwrap())
        } else if args.fields.is_none() && args.name.contains("/") {
            // If there were no fields provided, then assume that this will be
            // whatever was provided after the `/`.
            // For instance, `path/to/Setting/Dark` would create a Dark field
            // for the Setting table.
            Path::new(&args.name)
                .components()
                .last()
                .ok_or(Error::FailedToHandleComponent("Could not detect the field."))
                .map(|part| vec![part.as_os_str().to_string_lossy().to_string()])
        } else if schema == SchemaType::Schemafull {
            // At this point, it means that no fields were provided, but the
            // table will be schemafull so no need to create sub structs.
            // By default, a schemafull table will just have a `value` field.
            Ok(vec!["value".to_string()])
        } else {
            Err(Error::FailedToHandleComponent("No fields detected."))
        }?;
    let fields = fields.into_iter()
        .map(|field| ComponentName(field))
        .collect::<Vec<ComponentName>>();

    // Create the model.rs file if it does not exist.
    let first_path = component_path_no_ext.components().skip(2).next()
        .ok_or(Error::FailedToHandleComponent("Failed to get folder name for model mod."))?;
    let first_path = ComponentName(first_path.as_os_str().to_string_lossy().to_string());

    // Register the new model into the model.rs file.
    let model_contents = if base_path.with_extension("rs").exists() {
            let mut model_contents = fs::read_to_string(&base_path.with_extension("rs"))?;

            let mod_string = format!("pub mod {};", first_path.snake_case());
            if !model_contents.contains(&mod_string) {
                model_contents = append_after_marker(model_contents, "pub mod", mod_string);
            }

            model_contents
        } else {
            let mut model_contents = format!("pub mod {};\n\n", first_path.snake_case());
            model_contents.push_str(&include_str!("../stubs/model.txt"));
            model_contents
        };
    drop(first_path);
    fs::write(base_path.with_extension("rs"), model_contents)?;
    fs::create_dir_all(component_path.parent().unwrap())?;

    // Keep track of the full path.
    let mut path_split_cur = component_path_no_ext.components().take(3).collect::<PathBuf>();
    // Skip the `src` and `model` directories.
    let path_splits = component_path_no_ext.components().skip(3);

    for path_split in path_splits {
        let path_comp_name: ComponentName = path_split.into();
        path_split_cur.push(path_comp_name.snake_case());

        let path_split_contents = if path_split_cur.exists() {
                fs::read_to_string(&path_split_cur)?
            } else {
                String::new()
            };

        // Create the mod file.
        let mod_string = format!("pub mod {};", path_comp_name.snake_case());
        let path_split_contents = append_after_marker(path_split_contents, "pub mod", mod_string);
        let path_split_contents = path_split_contents.trim().to_string();

        let path_split_cur_parent = path_split_cur
            .parent()
            .ok_or(Error::FailedToHandleComponent("Couldn't extract current path."))?;

        fs::create_dir_all(&path_split_cur_parent)?;
        fs::write(path_split_cur_parent.with_extension("rs"), path_split_contents)?;
    }

    // Now handle the creation of the final struct itself.
    // Replace holding values with the actual size of the model.
    match schema {
        SchemaType::Schemafull => {
            let component_contents = if component_path.exists() {
                let component_contents = fs::read_to_string(&component_path)?;
                // Add the `|fields|` marker in so that we can replace it
                // regardless of which branch we took.
                let component_contents = append_after_marker(component_contents, "pub id: Option<Thing>,", "|fields|".to_string());
                component_contents
            } else {
                include_str!("../stubs/components/model_schemafull.txt")
                    .replace("|TableNamePascal|", &table.pascal_case())
                    .replace("|table_name_snake|", &table.snake_case())
            };

            let fields = fields
                .into_iter()
                .map(|field| format!("    pub {}: String,", field.snake_case()))
                .collect::<Vec<String>>()
                .join("\n");
            let model_contents = component_contents.replace("|fields|", &fields);

            fs::write(&component_path, model_contents)?;
        },
        SchemaType::Schemaless => {
            for field in &fields {
                // Create the struct.
                let table_path = component_path.clone();
                let field_path = component_path.with_extension("").join(field.snake_case()).with_extension("rs");
                fs::create_dir_all(field_path.parent().unwrap())?;
                let model_contents = include_str!("../stubs/components/model_schemaless.txt")
                    .replace("|TableNamePascal|", &table.pascal_case())
                    .replace("|table_name_snake|", &table.snake_case())
                    .replace("|ModelNamePascal|", &field.pascal_case())
                    .replace("|model_name_snake|", &field.snake_case());
                fs::write(&field_path, model_contents)?;

                // Now export that in the table class.
                // This needs to match the stub.
                let mod_string = format!("mod {};", field.snake_case());
                let use_string = format!("pub use {}::{}{};", field.snake_case(), table.pascal_case(), field.pascal_case());
                let table_contents = if table_path.exists() {
                    let table_contents = fs::read_to_string(&table_path)?;
                    let table_contents = append_after_marker(table_contents, "mod", mod_string);
                    let table_contents = append_after_marker(table_contents, "pub use", use_string);
                    table_contents
                } else {
                    let mut table_contents = String::new();
                    table_contents.push_str(&format!("{}\n", mod_string));
                    table_contents.push_str("\n");
                    table_contents.push_str(&format!("{}\n", use_string));
                    table_contents
                };
                fs::write(table_path, table_contents)?;
            }
        },
    }

    Ok(())
}

fn handle_store(args: ComponentStoreArgs) -> Result<()> {
    let store_name = ComponentName(args.name);
    let base_path = Path::new("src/store");
    let store_path = base_path.with_extension("rs");
    let engine_path = base_path.join(args.engine.snake_case()).with_extension("rs");
    let store_component_path = engine_path.with_extension("").join(store_name.snake_case()).with_extension("rs");

    // Prevent the store being created if it already exists.
    // Just error out, don't try to append or anything; let the user decide.
    if store_component_path.try_exists()? {
        return Err(Error::StoreAlreadyExists(store_name.pascal_case()));
    }

    fs::create_dir_all(&store_component_path.parent().unwrap())?;

    // If there is already a file, then we will need to add the new struct in.
    // Otherwise, we just need to copy the stub.
    let store_content = if store_path.try_exists()? {
        let store_content = fs::read_to_string(&store_path)?;

        let mod_string = format!("pub mod {};", args.engine.snake_case());
        let store_content = append_after_marker(store_content, "pub mod", mod_string);

        let use_string = format!("pub use {}::{};", args.engine.snake_case(), store_name.0);
        let store_content = append_after_marker(store_content, "pub use", use_string);

        store_content
    } else {
        include_str!("../stubs/store.txt")
            .replace("|EngineNamePascal|", args.engine.pascal_case())
            .replace("|engine_name_snake|", args.engine.snake_case())
            .replace("|StoreNamePascal|", &store_name.pascal_case())
            .replace("|store_name_snake|", &store_name.snake_case())
    };
    fs::write(store_path, store_content)?;

    // Make the Engine mod file.
    let engine_content = if engine_path.try_exists()? {
        let engine_content = fs::read_to_string(&engine_path)?;

        let mod_string = format!("pub mod {};", store_name.snake_case());
        let engine_content = append_after_marker(engine_content, "pub mod", mod_string);

        engine_content
    } else {
        include_str!("../stubs/store_engine.txt")
            .replace("|EngineNamePascal|", args.engine.pascal_case())
            .replace("|engine_name_snake|", args.engine.snake_case())
            .replace("|StoreNamePascal|", &store_name.pascal_case())
            .replace("|store_name_snake|", &store_name.snake_case())
    };
    fs::write(engine_path, engine_content)?;

    let store_component_content = match args.engine {
        Engine::Mem => include_str!("../stubs/components/store.txt"),
        _ => include_str!("../stubs/components/store_with_path.txt"),
    };
    let store_component_content = store_component_content
        .replace("|EngineNamePascal|", args.engine.pascal_case())
        .replace("|engine_name_snake|", args.engine.snake_case())
        .replace("|StoreNamePascal|", &store_name.pascal_case())
        .replace("|store_name_snake|", &store_name.snake_case());

    fs::write(store_component_path, store_component_content)?;

    Ok(())
}
