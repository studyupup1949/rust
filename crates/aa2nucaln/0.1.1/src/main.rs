// A rust implementation of https://academic.oup.com/bioinformatics/article/31/11/1836/2365396
// Convert an amino acid alignment to a nucleotide alignment for HMM training.
// Prints to stdout only, nothing fancy.

use aa2nucaln::is_stdin;
use aa2nucaln::writer;
use anyhow::bail;
use anyhow::Result;
use bio::io::fasta;
use clap::{Arg, Command};
use std::io;

fn main() -> Result<()> {
    // command line options
    let matches = Command::new("aa2nucaln")
        .version(clap::crate_version!())
        .author("Max Brown <mb39@sanger.ac.uk>")
        // .arg_required_else_help(true)
        .about("Convert an amino acid alignment to a nucelotide alignment.\nFor more info, see: https://academic.oup.com/bioinformatics/article/31/11/1836/2365396")
        .arg(
            Arg::new("fasta")
                .help("The reference amino acid alignment file in fasta format."),
        )
        .arg(
            Arg::new("ignore")
                .short('i')
                .long("ignore")
                .takes_value(true)
                .required(false)
                .default_value("true")
                .help("Should the lower-case letters in an amino acid alignment be ignored?"),
        )
        .get_matches();
        
    // parse command line options
    let input_fasta = matches.value_of("fasta");
    let ignore = matches.value_of_t("ignore")?;

    match input_fasta {
        Some(f) => {
            // read in the fasta.
            let fasta_reader = fasta::Reader::from_file(f)?;
            writer::write_faa(fasta_reader, ignore)?;
        }
        None => match is_stdin() {
            true => {
                let fasta_reader = fasta::Reader::new(io::stdin());
                writer::write_faa(fasta_reader, ignore)?;
            }
            false => bail!("There was no input from STDIN."),
        },
    }

    Ok(())
}
