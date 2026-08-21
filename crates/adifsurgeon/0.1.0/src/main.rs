use chrono::{DateTime, Utc};
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use std::process;

use adifsurgeon::parse_timestamp;

mod transforms;
use transforms::process_deletes;
use transforms::process_drops;
use transforms::process_inserts;
use transforms::process_keeps;
use transforms::process_replaces;
use transforms::process_times;

mod adif;
use adif::{get_header, parse_header, parse_record, parse_records, write_header, write_record};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "ADIFSurgeon is a tool for manipulating ADI files",
    after_help = "The order of operations is as follows:

* drop / keep entire records
* filter by time
* insert new values into each record
* replace values in existing records
* delete values from records
"
)]
struct Args {
    /// Display verbose output
    #[arg(short, long)]
    verbose: bool,

    /// List of input files
    #[arg(short, long, value_parser, num_args = 1..)]
    infile: Vec<PathBuf>,

    /// A single output file
    #[arg(short, long, value_parser)]
    outfile: PathBuf,

    /// Records to drop. Mutually exclusive with keep
    #[arg(short = 'D', long, value_parser = parse_key_val, num_args = 1..)]
    drop: Option<Vec<(String, String)>>,

    /// Records to keep. Mutually exclusive with drop
    #[arg(short, long, value_parser = parse_key_val, num_args = 1..)]
    keep: Option<Vec<(String, String)>>,

    /// Keys / values to insert into each record
    #[arg(short = 'I', long, value_parser = parse_key_val, num_args = 1..)]
    insert: Option<Vec<(String, String)>>,

    /// Keys / values to replace in each record
    #[arg(short, long, value_parser = parse_key_val, num_args = 1..)]
    replace: Option<Vec<(String, String)>>,

    /// Keys to delete from each record
    #[arg(short, long)]
    delete: Option<Vec<String>>,

    /// Drop any records before this timestamp. Time is UTC, formatted in YYYYMMDD or YYYYMMDDhhmmss.
    /// Time defaults to 00:00:00 if not specified.
    #[arg(short, long, value_parser = parse_timestamp)]
    start: Option<DateTime<Utc>>,

    /// Drop any records after this timestamp. Time is UTC, formatted in YYYYMMDD or YYYYMMDDhhmmss.
    /// Time defaults to 00:00:00 if not specified.
    #[arg(short, long, value_parser = parse_timestamp)]
    end: Option<DateTime<Utc>>,
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("Verbose mode enabled");
    }

    if args.drop.is_some() && args.keep.is_some() {
        println!("Error: --drop and --keep are mututally exclusive");
        process::exit(1);
    }

    let mut parsed_records: Vec<HashMap<String, String>> = Vec::new();

    // Collect all of the files into one set of parsed records
    for path in args.infile.iter() {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let _header = parse_header(&mut reader)?;
        let records = parse_records(&mut reader)?;

        for record in records.iter() {
            let cooked = parse_record(record);
            println!("Record:\n{:#?}\n", cooked);
            parsed_records.push(cooked);
        }
    }

    // Remove any records meant to be dropped
    let filtered_records = process_drops(&parsed_records, args.drop);

    // Remove any unneeded records
    let kept_records = process_keeps(&filtered_records, args.keep);

    // Remove any records outside of the allotted time
    let time_filtered_records = process_times(&kept_records, args.start, args.end);

    // Insert any new records
    let added_records = process_inserts(&time_filtered_records, args.insert);

    // Replace any existing records with new values
    let doctored_records = process_replaces(&added_records, args.replace);

    // Delete any records
    let deleted_records = process_deletes(&doctored_records, args.delete);

    // Write the result
    let outfile = File::create(args.outfile)?;
    let mut writer = BufWriter::new(outfile);

    write_header(&get_header(), &mut writer)?;
    for record in deleted_records.iter() {
        write_record(record, &mut writer)?;
    }

    Ok(())
}
