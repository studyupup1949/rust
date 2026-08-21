//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later
//! Main CLI program for converting Alfa source files to XACML 3.0.
use a2x::args::CLIArgs;
use a2x::context::Config;
use a2x::context::Context;
use a2x::AlfaFile;
use clap::Parser;
use log::info;
use log::warn;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;
use walkdir::WalkDir;

fn main() -> ExitCode {
    let args = CLIArgs::parse();
    env_logger::init();
    // if requested, just output the built-in definitions in ALFA format.
    if args.show_builtins {
        info!("showing builtins");
        // create a new context
        let ctx = Context::default();
        //
        let mut stdout = io::stdout();
        let _ = ctx.serialize_builtins(&mut stdout);
        return ExitCode::SUCCESS;
    }
    if let Some(outdir) = &args.output_dir {
        // input paths that we should attempt to parse
        let input_paths: Vec<PathBuf> = get_input_paths(&args.alfa_dir);
        // define a configuration for the conversion
        let ctx = Rc::new(Context::new(Config {
            base_namespace: args.base_namespace,
            enable_builtins: !args.disable_builtins,
            version: Some("1.0".to_string()),
        }));
        // get alfa file contents
        let alfa_sources: Vec<AlfaFile> = get_alfa_sources(input_paths);
        // Now that we have the Alfa source files as strings, compile them all.
        let xfilesres = a2x::alfa_compile(&ctx, alfa_sources);
        // check the result
        match xfilesres {
            Err(pe) => {
                warn!("compilation failed");
                eprintln!("Conversion to XACML Failed:");
                eprintln!("{pe}");
                return ExitCode::FAILURE;
            }
            Ok(xfiles) => {
                eprintln!("Compiled to {} policies.", xfiles.len());
                let policy_output_path = Path::new(outdir);
                for x in xfiles {
                    let r = a2x::write_xentry(policy_output_path, &x);
                    info!("result of writing policy: {r:?}");
                }
            }
        }
    } else {
        println!("Provide an output directory for XACML policies using the --output option.");
    }
    return ExitCode::SUCCESS;
}

/// Read a list of paths into file contents.
///
/// # Arguments
/// * `input` - A vector of path names to alfa source files.
///
/// # Panics
///
/// The function panics if any file could not be opened or read.
fn get_alfa_sources(input: Vec<PathBuf>) -> Vec<AlfaFile> {
    let mut alfa_sources: Vec<AlfaFile> = vec![];
    for i in input {
        info!("Alfa path to parse: {}", i.display());
        // read file to string
        let mut f = std::fs::File::open(&i).expect("could not open file");
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).expect("could not read file");
        alfa_sources.push(AlfaFile {
            filename: i.clone(),
            contents: buffer,
        });
    }
    alfa_sources
}

/// Expand a set of paths into all the child files ending in ".alfa".
fn get_input_paths(args: &Vec<String>) -> Vec<PathBuf> {
    info!("input paths: {args:?}");
    let suffix = "alfa";
    // input paths that we should attempt to parse
    let mut input_paths: Vec<PathBuf> = vec![];
    // loop through args, and add any paths that exist (error on non-existing paths)
    for a in args {
        let p = Path::new(&a);
        if p.is_dir() {
            for entry in WalkDir::new(p)
                .into_iter()
                .filter_map(std::result::Result::ok)
            {
                if let Some(ext) = entry.path().extension() {
                    if ext == suffix {
                        input_paths.push(entry.path().to_path_buf());
                    }
                }
            }
        } else if p.is_file() {
            if let Some(ext) = p.extension() {
                if ext == suffix {
                    input_paths.push(p.to_path_buf());
                }
            }
        }
    }
    input_paths
}
