mod commands;
mod config;

use clap::{Parser, Subcommand};
use commands::login::login;
use commands::app;
use commands::world;

#[derive(Parser)]
#[command(name = "abpilot")]
#[command(about = "ABPilot CLI Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Login to ABPilot
    Login,
    /// Manage apps
    App {
        #[command(subcommand)]
        action: AppCommands,
    },
    /// Manage worlds
    World {
        #[command(subcommand)]
        action: WorldCommands,
    },
}

#[derive(Subcommand)]
enum AppCommands {
    /// List all apps
    List,
    /// Create a new app
    Create {
        /// Name for the app
        name: String,
    },
    /// Delete an app
    Delete {
        /// App ID to delete
        app_id: String,
    },
    /// Reset app secret
    ResetSecret {
        /// App ID
        app_id: String,
    },
    /// Upload files to app
    Upload {
        /// App ID
        app_id: String,
        /// Files to upload
        files: Vec<String>,
    },
    /// Download files from app
    Download {
        /// App ID
        app_id: String,
        /// Files to download
        files: Vec<String>,
    },
}

#[derive(Subcommand)]
enum WorldCommands {
    /// List all worlds
    List,
    /// Create a new world
    Create {
        /// Name for the world
        name: String,
    },
    /// Delete a world
    Delete {
        /// World ID to delete
        world_id: String,
    },
    /// Get world details
    Get {
        /// World ID
        world_id: String,
    },
    /// Reset world secret
    ResetSecret {
        /// World ID
        world_id: String,
    },
    /// Upload files to world
    Upload {
        /// World ID
        world_id: String,
        /// Files to upload
        files: Vec<String>,
    },
    /// Download files from world
    Download {
        /// World ID
        world_id: String,
        /// Files to download
        files: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login => {
            if let Err(e) = login().await {
                eprintln!("Login failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::App { action } => {
            let result = match action {
                AppCommands::List => app::list().await,
                AppCommands::Create { name } => app::create(name).await,
                AppCommands::Delete { app_id } => app::delete(app_id).await,
                AppCommands::ResetSecret { app_id } => app::reset_secret(app_id).await,
                AppCommands::Upload { app_id, files } => app::upload(app_id, files).await,
                AppCommands::Download { app_id, files } => app::download(app_id, files).await,
            };
            
            if let Err(e) = result {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::World { action } => {
            let result = match action {
                WorldCommands::List => world::list().await,
                WorldCommands::Create { name } => world::create(name).await,
                WorldCommands::Delete { world_id } => world::delete(world_id).await,
                WorldCommands::Get { world_id } => world::get(world_id).await,
                WorldCommands::ResetSecret { world_id } => world::reset_secret(world_id).await,
                WorldCommands::Upload { world_id, files } => world::upload(world_id, files).await,
                WorldCommands::Download { world_id, files } => world::download(world_id, files).await,
            };
            
            if let Err(e) = result {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
