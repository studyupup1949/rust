extern crate glob;
extern crate proc_macro;
use quote::quote;
use providers::setup_providers_impl;
use commands::setup_commands_impl;
use routes::setup_routes_impl;

mod providers;
mod commands;
mod routes;


#[proc_macro]
pub fn setup_commands(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    setup_commands_impl(tokens)
}

#[proc_macro]
pub fn setup_routes(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    setup_routes_impl(tokens)
}

#[proc_macro]
pub fn setup_providers(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    setup_providers_impl(tokens)
}

#[proc_macro_attribute]
pub fn main(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: syn::ItemFn = syn::parse(item.clone()).unwrap();


    quote! {
    #[macro_use] extern crate rocket;
    use adrift::tracing_subscriber;
    use std::collections::HashMap;
    use std::any::Any;
    use adrift::clap;
    use adrift::dotenv;
    use adrift::once_cell;
    use adrift::clap::error::KindFormatter;
    pub mod commands {
        adrift::macros::setup_commands!();
    }

    pub mod routes {
       adrift::macros::setup_routes!();
    }

    pub mod providers {
        adrift::macros::setup_providers!();
     }

    #input

    #[adrift::tokio::main(crate = "adrift::tokio")]
    async fn main() {
        dotenv::dotenv().ok();
        
        tracing_subscriber::fmt()
        // .json()
        .init();

        let mut command = clap::Command::new("Cli");
        let commands = commands::COMMANDS;
        let core_commands = adrift::core::commands::get_commands();

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

        adrift::Container::singleton(&|_| adrift::RoutesConfig {
            inner: routes::ROUTES.items.clone(),
        }).expect("Failed to register routes");

        adrift::Container::singleton(&|_| adrift::TemplateConfig {
            functions: HashMap::new(),
        }).expect("Failed to register routes");
        
        let core_providers = adrift::core::service_providers::get_providers();

        for provider in &core_providers {
            provider.register().await;
        }

        for provider in &core_providers {
            provider.boot().await;
        }

        for provider in &providers::PROVIDERS.items {
            provider.register().await;
        }

        for provider in &providers::PROVIDERS.items {
            provider.boot().await;
        }

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
                            clap::ArgAction::Count => {
                                if let Some(value) = subcommand.1.get_one::<u8>(key) {
                                    args.insert(key.to_string(), value.to_owned().to_string());
                                }
                            },
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
