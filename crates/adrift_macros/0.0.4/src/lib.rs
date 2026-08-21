extern crate glob;
extern crate proc_macro;
use std::env;
use std::fs::File;
use std::io::Read;

use convert_case::{Case, Casing};
use glob::glob;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;
use syn::parse::Parse;
use syn::{parse_macro_input, Ident, LitStr, Token, Type};

struct LazyLoadTraitsInput {
    pub name: Ident,
    pub trait_type: Type,
    pub path: LitStr,
}

impl Parse for LazyLoadTraitsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let trait_type: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let path: LitStr = input.parse()?;

        Ok(LazyLoadTraitsInput {
            name,
            trait_type,
            path,
        })
    }
}

#[proc_macro]
pub fn lazy_load_traits(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let LazyLoadTraitsInput {
        name,
        trait_type,
        path,
    } = parse_macro_input!(tokens as LazyLoadTraitsInput);

    let glob_path = if path.value().starts_with('/') {
        format!("{}/*.rs", path.value())
    } else {
        format!(
            "{}/src/{}/*.rs",
            env::var("CARGO_MANIFEST_DIR").unwrap(),
            path.value()
        )
    };

    let mut files = vec![];

    for entry in glob(&glob_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let file_name = path
                    .clone()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                files.push((file_name.clone(), file_name.to_case(Case::Pascal)));
            }
            Err(_) => {}
        }
    }

    let mut items = TokenStream::new();
    let mut mods = TokenStream::new();

    files.iter().for_each(|(file, struc)| {
        let a = format_ident!("{}", file);
        let b = format_ident!("{}", struc);

        let token: TokenStream = quote!(Box::new(#a::#b),);

        items.extend(token);

        let token: TokenStream = quote!(pub mod #a;);

        mods.extend(token);
    });

    quote! {
        #mods

        pub struct Traits {
            pub items: Vec<Box<dyn #trait_type>>,
        }

       pub const #name: adrift::once_cell::sync::Lazy<Traits> = adrift::once_cell::sync::Lazy::new(|| Traits {
            items: vec![
                #items
            ]
       });
    }
    .into()
}

#[proc_macro]
pub fn lazy_load_commands(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let glob_path = format!(
        "{}/src/{}/*.rs",
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        "commands"
    );

    let mut files = vec![];

    for entry in glob(&glob_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let file_name = path
                    .clone()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                files.push((file_name.clone(), file_name.to_case(Case::Pascal)));
            }
            Err(_) => {}
        }
    }

    let mut items = TokenStream::new();
    let mut mods = TokenStream::new();

    files.iter().for_each(|(file, struc)| {
        let a = format_ident!("{}", file);
        let b = format_ident!("{}", struc);

        let token: TokenStream = quote!(Box::new(#a::#b),);

        items.extend(token);

        let token: TokenStream = quote!(pub mod #a;);

        mods.extend(token);
    });

    quote! {
        #mods

        pub struct Traits {
            pub items: Vec<Box<dyn Command>>,
        }

       pub const COMMANDS: adrift::once_cell::sync::Lazy<Traits> = adrift::once_cell::sync::Lazy::new(|| Traits {
            items: vec![
                #items
            ]
       });
    }
    .into()
}

#[proc_macro]
pub fn setup_commands(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let glob_path = format!(
        "{}/src/{}/*.rs",
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        "commands"
    );

    let mut files = vec![];

    for entry in glob(&glob_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let file_name = path
                    .clone()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                files.push((file_name.clone(), file_name.to_case(Case::Pascal)));
            }
            _ => (),
        }
    }

    let mut items = TokenStream::new();
    let mut mods = TokenStream::new();

    files.iter().for_each(|(file, struc)| {
        let a = format_ident!("{}", file);
        let b = format_ident!("{}", struc);

        let token: TokenStream = quote!(Box::new(#a::#b),);

        items.extend(token);

        let token: TokenStream = quote!(pub mod #a;);

        mods.extend(token);
    });

    quote! {
        use adrift::commands::{Commands, Command};
        #mods

        #[allow(clippy::all)] pub const COMMANDS: adrift::once_cell::sync::Lazy<Commands> = adrift::once_cell::sync::Lazy::new(|| Commands {
            items: vec![
                #items
            ]
       });
    }
    .into()
}

#[proc_macro]
pub fn setup_routes(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let glob_path = format!(
        "{}/src/{}/*.rs",
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        "routes"
    );

    let mut files = vec![];

    for entry in glob(&glob_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let file_name = path
                    .clone()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                if let Ok(file) = &mut File::open(path) {
                    let mut contents = String::new();

                    // read the contents of the file into the string
                    if file.read_to_string(&mut contents).is_ok() {
                        files.push((file_name.clone(), contents));
                    }
                }
            }
            Err(_) => {}
        }
    }

    let method_regex = Regex::new(r"#\[get.+]\s*?pub fn\s+([^(]+)").unwrap();

    let mut items = TokenStream::new();
    let mut mods = TokenStream::new();

    files.iter().for_each(|(file, file_content)| {
        let a = format_ident!("{}", file);

        let token: TokenStream = quote!(pub mod #a;);

        mods.extend(token);

        let mut functions = Vec::new();

        for caps in method_regex.captures_iter(file_content) {
            let function_name = caps.get(1).unwrap().as_str();
            functions.push(function_name);
        }

        // print the function names
        for function in functions {
            let fun = format_ident!("{}", function);

            let token: TokenStream = quote!(#a::#fun,);
            items.extend(token);
        }
    });

    let tokens = quote! {
        #mods

        pub struct Routes {
            pub items: Vec<rocket::Route>,
        }

        #[allow(clippy::all)] pub const ROUTES: adrift::once_cell::sync::Lazy<Routes> = adrift::once_cell::sync::Lazy::new(|| Routes {
            items: rocket::routes![
                #items
            ]
       });
    };

    tokens.into()
}

#[proc_macro_attribute]
pub fn main(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: syn::ItemFn = syn::parse(item.clone()).unwrap();


    quote! {
    #[macro_use] extern crate rocket;
    use std::collections::HashMap;
    use std::any::Any;
    use adrift::clap;
    use adrift::once_cell;
    use adrift::clap::error::KindFormatter;
    pub mod commands {
        adrift::macros::setup_commands!();
    }

    pub mod routes {
       adrift::macros::setup_routes!();
    }

    #input

    #[adrift::tokio::main(crate = "adrift::tokio")]
    async fn main() {
        let mut command = clap::Command::new("Cli");
        let commands = commands::COMMANDS;
        let core_commands = adrift::core::get_commands();

        for cmd in &commands.items {
            let name = cmd.name();
            let subcommand = clap::Command::new(name)
                .about(cmd.description())
                .args(cmd.args());

            command = command.subcommand(subcommand);
        }

        for cmd in &core_commands {
            let name = cmd.name();
            let subcommand = clap::Command::new(name)
                .about(cmd.description())
                .args(cmd.args());

            command = command.subcommand(subcommand);
        }

        let matches = command
            .clone()
            .try_get_matches()
            .map_err(|e| e.apply::<KindFormatter>())
            .unwrap_or_else(|e| e.exit());

        adrift::Container::bind(|_| adrift::Routes {
            inner: routes::ROUTES.items.clone(),
        }).expect("Failed to register routes");

        boot().await;

        if let Some(subcommand) = matches.subcommand() {
            let cmd = &commands
                .items
                .iter()
                .find(move |c| c.name() == subcommand.0);

           let fcmd : std::option::Option<&Box<dyn adrift::commands::Command>> =  match cmd {
                Some(cmd) => Some(cmd),
                None => {
                    core_commands
                    .iter()
                    .find(move |c| c.name() == subcommand.0)
                },
            };

            match fcmd {
                Some(c) => {
                    let mut args = HashMap::new();

                    for arg in c.args() {
                        let key = arg.get_id().as_str();

                        match arg.get_action() {
                            // clap::ArgAction::Set => todo!(),
                            // clap::ArgAction::Append => todo!(),
                            clap::ArgAction::SetTrue => {
                                if let Some(value) = subcommand.1.get_one::<bool>(key) {
                                    args.insert(key.to_string(), value.to_owned().to_string());
                                }
                            }
                            clap::ArgAction::SetFalse => {
                                if let Some(value) = subcommand.1.get_one::<bool>(key) {
                                    args.insert(key.to_string(), value.to_owned().to_string());
                                }
                            }
                            // clap::ArgAction::Count => todo!(),
                            // clap::ArgAction::Help => todo!(),
                            // clap::ArgAction::Version => todo!(),
                            _ => {
                                if let Some(value) = subcommand.1.get_one::<String>(key) {
                                    args.insert(key.to_string(), value.to_owned());
                                }
                            }
                        }
                    }

                    c.handle(args).await.unwrap();

                    if c.require_full_rebuild() {
                        std::process::Command::new("cargo").args(["clean"]).output();
                    } else if c.require_rebuild() {
                        std::process::Command::new("cargo").args(["clean", "-p", "adrift_macros"]).output();
                    }
                }
                None => todo!(),
            }
        } else {
            command.print_help().unwrap();
        }
    }
    }
    .into()
}
