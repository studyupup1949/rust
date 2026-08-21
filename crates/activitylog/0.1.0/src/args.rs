//! This module provides representation of the CLI app arguments.
//! Each of these structs and enums is implementing and deriving the [`clap`] library traits.
//! 
//! Here is the list of commands provided to the user (at the moment this documentation piece is written) :
//! - activitylog `commit` <message> [--section <section-name>]
//! - activitylog `save`
//! - activitylog `switch` <subject-name>
//! - activitylog `subject` <list|add|remove|update>
//! - activitylog `convert` <output-format>

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Csv,
    Json,
    Xml
}

#[derive(Debug, Subcommand, Clone)]
pub enum Subject {
    /// List all subjects 
    List {
        /// Regex pattern to match during search
        #[arg(short, long)]
        filter: Option<String>
    },
    /// Add a subject to the list
    #[command(arg_required_else_help = true)]
    Add {
        /// Name of the subject to add
        name: String
    },
    /// Remove a specified subject
    #[command(arg_required_else_help = true)]
    Remove {
        /// Name of the subject to remove from the list
        name: String,
    },
    /// Update the name of a specified subject
    #[command(arg_required_else_help = true)]
    Update {
        /// Name of the subject to update
        name: String,
        /// New name
        new: String
    }
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    /// Commit a new task in the history (as temporary logs)
    #[command(arg_required_else_help = true)]
    Commit {
        /// Description of the task you are about to start or complete
        title: String,
        
        /// Section name, in the current subject, you are working on
        #[arg(short, long)]
        section: Option<String>
    },
    
    /// Save temporary commits in the actual history
    Save,

    /// Switch to another subject to work on
    #[command(arg_required_else_help = true)]
    Switch {
        /// Subject name you are switching to
        subject: String
    },

    /// Subcommands related to subjects defined in the config information file
    #[command(arg_required_else_help = true)]
    Subject {
        #[command(subcommand)]
        command: Subject
    },
    /// Convert history into another format
    #[command(arg_required_else_help = true)]
    Convert {
        /// Output format
        format: Format,
        /// Wether all the history is the converted, only the latest file by default
        #[arg(short, long, default_value_t = false)]
        all: bool,
        /// If all history considered merge the conversion result, separated files by default
        #[arg(short, long, default_value_t = false)]
        merge: bool,
    },
    /// Generate a sample set of history record
    Generate
}

#[derive(Parser, Debug)]
#[command(version = "1.0", author = "Thierry Kunda", arg_required_else_help = true)]
pub struct Args {
    #[command(subcommand)]
    pub commands: Command
}