mod abi;
mod converter;
mod file_ops;
mod json_parser;
mod tests;

use converter::Converter;
use file_ops::{convert_directory, convert_file, convert_stdin_to_stdout, ConvertOptions};
use std::env;
use std::path::Path;
use std::process;

const VERSION: &str = "1.0.2";

fn print_help() {
    println!(
        r#"
abi2human v{VERSION} - Convert Ethereum ABI to human-readable format
Optimized for AI agents to efficiently consume smart contract interfaces

USAGE:
  abi2human [options] <input> [output]

ARGUMENTS:
  input    Input ABI file (.json) or directory
  output   Output file or directory (optional)

OPTIONS:
  -o, --stdout     Output to stdout
  -r, --raw        Output raw text format instead of JSON
  -h, --help       Show this help message
  -v, --version    Show version
  -q, --quiet      Suppress non-output messages
  -d, --dir        Process directory
  -p, --pattern    Glob pattern for filtering files
  -s, --suffix     Custom suffix for output files (default: ".readable")
  --no-pretty      Disable pretty-printing

EXAMPLES:
  # Quick ABI inspection
  abi2human contract.json -o
  
  # Raw text format
  abi2human contract.json -or
  
  # Convert and save file
  abi2human contract.json output.json
  
  # Batch convert directory
  abi2human ./abis/ -d ./readable/

FOR AI AGENTS:
  This tool helps you read Ethereum ABIs efficiently without consuming excessive tokens.
"#
    );
}

struct CliArgs {
    input: Option<String>,
    output: Option<String>,
    stdout: bool,
    raw: bool,
    quiet: bool,
    directory: bool,
    pattern: Option<String>,
    suffix: String,
    pretty: bool,
    help: bool,
    version: bool,
}

impl CliArgs {
    fn parse() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut cli_args = CliArgs {
            input: None,
            output: None,
            stdout: false,
            raw: false,
            quiet: false,
            directory: false,
            pattern: None,
            suffix: ".readable".to_string(),
            pretty: true,
            help: false,
            version: false,
        };

        let mut i = 1;
        let mut positionals = Vec::new();

        while i < args.len() {
            let arg = &args[i];

            if arg.starts_with('-') {
                match arg.as_str() {
                    "-h" | "--help" => cli_args.help = true,
                    "-v" | "--version" => cli_args.version = true,
                    "-o" | "--stdout" => cli_args.stdout = true,
                    "-r" | "--raw" => cli_args.raw = true,
                    "-q" | "--quiet" => cli_args.quiet = true,
                    "-d" | "--dir" => cli_args.directory = true,
                    "--no-pretty" => cli_args.pretty = false,
                    "-p" | "--pattern" => {
                        i += 1;
                        if i < args.len() {
                            cli_args.pattern = Some(args[i].clone());
                        }
                    }
                    "-s" | "--suffix" => {
                        i += 1;
                        if i < args.len() {
                            cli_args.suffix = args[i].clone();
                        }
                    }
                    "-or" | "-ro" => {
                        cli_args.stdout = true;
                        cli_args.raw = true;
                    }
                    _ => {}
                }
            } else {
                positionals.push(arg.clone());
            }
            i += 1;
        }

        if !positionals.is_empty() {
            cli_args.input = Some(positionals[0].clone());
        }
        if positionals.len() > 1 {
            cli_args.output = Some(positionals[1].clone());
        }

        cli_args
    }
}

fn main() {
    let args = CliArgs::parse();

    if args.help {
        print_help();
        process::exit(0);
    }

    if args.version {
        println!("abi2human v{VERSION}");
        process::exit(0);
    }

    let log = |msg: &str| {
        if !args.quiet {
            eprintln!("{msg}");
        }
    };

    if let Some(input) = args.input {
        let input_path = Path::new(&input);

        if !input_path.exists() {
            eprintln!("Error: Input path '{input}' does not exist");
            process::exit(1);
        }

        let options = ConvertOptions {
            suffix: args.suffix,
            pretty: args.pretty,
            pattern: args.pattern,
        };

        if input_path.is_dir() {
            let output_dir = if let Some(output) = args.output {
                Path::new(&output).to_path_buf()
            } else {
                input_path.join("readable")
            };

            log(&format!(
                "🔄 Converting ABI files from {} to {}",
                input_path.display(),
                output_dir.display()
            ));

            let results = convert_directory(input_path, &output_dir, &options);

            let successful: Vec<_> = results.iter().filter(|r| r.success).collect();
            let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();

            if !successful.is_empty() {
                log(&format!(
                    "✅ Successfully converted {} files",
                    successful.len()
                ));
            }

            if !failed.is_empty() {
                eprintln!("❌ Failed to convert {} files:", failed.len());
                for result in failed {
                    if let Some(error) = &result.error {
                        eprintln!("  - {}: {}", result.input_path.display(), error);
                    }
                }
                process::exit(1);
            }
        } else if args.stdout {
            let content = match std::fs::read_to_string(input_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading file: {e}");
                    process::exit(1);
                }
            };

            let abi_items = match Converter::parse_abi_content(&content) {
                Ok(items) => items,
                Err(e) => {
                    eprintln!("Error parsing ABI: {e}");
                    process::exit(1);
                }
            };

            if abi_items.is_empty() {
                eprintln!("Error: No valid ABI found in file");
                process::exit(1);
            }

            let human_readable = Converter::convert_to_human_readable(&abi_items);

            if args.raw {
                for item in human_readable {
                    println!("{item}");
                }
            } else {
                let formatted = Converter::format_as_json_array(&human_readable, args.pretty);
                print!("{formatted}");
                if args.pretty {
                    println!();
                }
            }
        } else {
            let output_path = args.output.as_ref().map(Path::new);
            let result = convert_file(input_path, output_path, &options);

            if result.success {
                if let Some(output) = result.output_path {
                    log(&format!(
                        "✅ Converted {} → {} ({} items)",
                        result.input_path.display(),
                        output.display(),
                        result.item_count.unwrap_or(0)
                    ));
                }
            } else {
                if let Some(error) = result.error {
                    eprintln!("❌ Error: {error}");
                }
                process::exit(1);
            }
        }
    } else {
        let options = ConvertOptions::default();
        if let Err(e) = convert_stdin_to_stdout(&options) {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
