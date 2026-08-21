//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later
//! Main CLI program for converting Alfa source files to XACML 3.0.
use a2x::AlfaFile;
use a2x::args::CLIArgs;
use a2x::context::Config;
use a2x::context::Context;
use clap::Parser;
use log::{info, warn};
use miette::Report;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;
use std::time;
use walkdir::WalkDir;

fn main() -> ExitCode {
    let program_start = time::Instant::now();
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
    print_program_header();
    if let Some(outdir) = &args.output_dir {
        // Print out the input files/paths, and output path
        for ip in &args.alfa_dir {
            eprintln!("Input:  {}", ip);
        }
        eprintln!("Output: {}", outdir);
        eprintln!();
        eprintln!("Scanning input directory...");
        // input paths that we should attempt to parse
        let input_paths: Vec<PathBuf> = get_input_paths(&args.alfa_dir);
        eprintln!("Found {} ALFA policy files", input_paths.len());
        eprintln!();
        // define a configuration for the conversion
        let ctx = Rc::new(Context::new(Config {
            base_namespace: args.base_namespace,
            enable_builtins: !args.disable_builtins,
            version: Some("1.0".to_string()),
        }));
        // get alfa file contents
        let alfa_sources: Vec<AlfaFile> = get_alfa_sources(input_paths);
        let alfa_sources_count = alfa_sources.len();
        // Now that we have the Alfa source files as strings, compile them all.
        let xfilesres = a2x::alfa_compile(&ctx, alfa_sources);
        eprintln!();
        let mut xacml_written = 0;
        let mut rules_written = 0;
        let mut policysets_written = 0;
        let mut policies_written = 0;
        // check the result
        match xfilesres {
            Err(pe) => {
                warn!("compilation of ALFA sources failed: {:?}", pe);
                eprintln!("Conversion to XACML Failed:");
                // If we checked for a PestParseError, and converted
                // the wrapped error using into_miette(), we could get
                // consistent miette output.  But as of pest 2.8.3,
                // the miette adapter does not preserve the path or
                // position in the file (the message is correct; but
                // the annotated line numbers always start at 0).
                //match pe {
                //    ParseError::PestParseError(ppe) => {
                //        eprintln!("{:?}", Report::new(ppe.into_miette()));
                //    } ...
                //}
                eprintln!("{:?}", Report::new(pe));
                return ExitCode::FAILURE;
            }
            Ok(xfiles) => {
                let policy_output_path = Path::new(outdir);
                eprintln!("Writing XACML policies:");
                for x in xfiles {
                    let write_res = a2x::write_xentry(policy_output_path, &x);
                    xacml_written += 1;
                    rules_written += x.rule_count();
                    policysets_written += x.policyset_count();
                    policies_written += x.policy_count();
                    // get details for policy
                    if let Ok(output_fn) = write_res {
                        eprintln!("  ✓ {}", output_fn);
                    } else {
                        eprintln!("Failed to write XACML policy: {write_res:?}");
                    }
                }
            }
        }
        eprintln!();
        eprintln!("Summary:");
        eprintln!("{}", "-".repeat(8));
        eprintln!("ALFA files processed  : {}", alfa_sources_count);
        eprintln!("XACML files generated : {}", xacml_written);
        eprintln!("Policy Sets written   : {}", policysets_written);
        eprintln!("Policies written      : {}", policies_written);
        eprintln!("Rules written         : {}", rules_written);
        eprintln!();
    } else {
        println!("Provide an output directory for XACML policies using the --output option.");
    }
    eprintln!("Total time: {}", format_duration(program_start.elapsed()));
    eprintln!();
    eprintln!("✓ Conversion completed successfully");
    ExitCode::SUCCESS
}

/// Print the program name and version, with a header separator and whitespace.
fn print_program_header() {
    let hdr_text = format!("ALFA to XACML Converter v{}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("{}", hdr_text);
    eprintln!("{}", "=".repeat(hdr_text.len()));
    eprintln!();
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
            filename: i.to_str().unwrap_or("<unknown path>").to_owned(),
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

/// Display a duration with a single digit after the decimal.
fn format_duration(duration: std::time::Duration) -> String {
    let millis = duration.as_secs_f64() * 1000.0;
    if millis < 1000.0 {
        format!("{:.1}ms", millis)
    } else {
        format!("{:.1}s", duration.as_secs_f64())
    }
}
