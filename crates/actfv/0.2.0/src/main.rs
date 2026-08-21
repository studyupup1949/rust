use std::error::Error;
use std::path::Path;
use std::time::Duration;

use clap::Parser;

use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::new_debouncer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source aws credentials file. Ex: ~/.aws/credentials
    #[arg()]
    source_file_path: String,

    /// Target file. Ex: mysecret.tfvars
    #[arg()]
    target_file_path: String,

    /// Specify the specific profile in the aws credentials file to load
    #[arg(short, long, default_value = "default")]
    profile: String,

    /// Watch the source file constantly for changes
    #[arg(short, long, default_value_t = false)]
    watch: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.watch {
        watch(
            &args.source_file_path,
            &args.target_file_path,
            &args.profile,
        )?;
    } else {
        actfv::parse_and_write(
            &args.source_file_path,
            &args.target_file_path,
            &args.profile,
        )?;
    }

    Ok(())
}

fn watch(
    source_file_path: &String,
    target_file_path: &String,
    profile: &String,
) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_secs(2), None, tx)?;

    debouncer
        .watcher()
        .watch(source_file_path.as_ref(), RecursiveMode::NonRecursive)?;

    debouncer.cache().add_root(
        <String as AsRef<Path>>::as_ref(source_file_path),
        RecursiveMode::NonRecursive,
    );

    for res in rx {
        match res {
            Ok(_) => {
                println!("Source file changed, updating target file.");
                actfv::parse_and_write(source_file_path, target_file_path, profile)?;
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
