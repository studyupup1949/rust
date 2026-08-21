/*
* Copyright (c) 2025 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENSE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use anyhow::anyhow;
use clap::Parser;
use colored::Colorize;
use hayagriva::io::from_biblatex_str;
use hayagriva::{BibliographyDriver, BibliographyRequest, CitationItem, CitationRequest, Entry};
use regex::Regex;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

mod citation_replacer;
pub mod utils;

use citation_replacer::CitationReplacer;
use utils::{check_args, setup_locale_styles, without_first};

/// Add bibliographies to markdown files.
///
/// Inserts html linked citations to the bibliography that is added to the bottom of the markdown.
/// The bibliographies added can be formatted with a chosen *.csl file. Although this does NOT
/// format the citation elements.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Bibtex files for creating the bibliography.
    #[arg(short, long, value_name = "FILE")]
    bib: PathBuf,

    /// CSL file to style citations.
    #[arg(short, long, value_name = "FILE")]
    csl: PathBuf,

    /// New markdown file to generate, with added bibliography.
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Perform dry run; no files are modified.
    #[arg(long)]
    dryrun: bool,

    /// Exit if any citation cannot be found.
    #[arg(long)]
    strict: bool,

    /// Print bibliography to terminal.
    #[arg(long)]
    term: bool,

    /// Turn off html links.
    #[arg(long)]
    nohtml: bool,

    /// Markdown file to add bibliography to.
    markdown: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    if args.dryrun {
        println!("{} {:?}", "[Reading from file]".green(), args.markdown);
    }
    // Produces struct version with no Options
    let checked_args = check_args(args)?;
    if checked_args.dryrun {
        println!("{} {:?}", "[Writing to file]".green(), checked_args.output);
    }

    if let Ok(contents) = fs::read_to_string(checked_args.markdown.clone()) {
        // Matches the following (across multiple lines):
        // @letters2025
        // [@letters2025]
        // [@letters2025, @moreletters2025]
        let re = Regex::new(r"(\[@[@A-Za-z0-9\n\s\t,]+\])|@([A-Za-z0-9]+)")
            .map_err(|_| anyhow!("Failed to compile regex for markdown citations."))?;
        let mut cite_replacer = CitationReplacer::new(checked_args.dryrun, checked_args.nohtml);
        let edited_markdown = re.replace_all(&contents, &mut cite_replacer);

        if checked_args.dryrun {
            println!("{}\n{:?}", "[Edited Markdown]".green(), edited_markdown);
        }

        // Setup bibliographies using csl and locales files using hayagriva
        let mut driver = BibliographyDriver::new();
        let bib = from_biblatex_str(&fs::read_to_string(checked_args.bib).unwrap()).unwrap();
        let mut items = Vec::<CitationItem<'_, Entry>>::new();
        for (key, _) in cite_replacer.citations {
            if let Some(entry) = bib.get(&key) {
                items.push(CitationItem::with_entry(entry));
            } else {
                if checked_args.strict {
                    return Err(anyhow!(
                        "Citation key, {}, could not be found. Exiting program as strict mode is on.",
                        key
                    ));
                }
                if checked_args.dryrun {
                    eprintln!(
                        "{} The citation key, {}, was not found in the bibliography.",
                        "[Error]".red(),
                        key
                    );
                }
            }
        }

        // Add citations to to driver
        let result = if let Some(csl_file) = checked_args.csl.to_str() {
            let local_path = include_str!("../assets/locales-en-US.xml");
            let (locales, styles) = setup_locale_styles(local_path, csl_file)?;
            driver.citation(CitationRequest::from_items(items, &styles, &locales));

            driver.finish(BibliographyRequest {
                style: &styles,
                locale: None,
                locale_files: &locales,
            })
        } else {
            Err(anyhow!(
                "The csl file, {:?}, is not valid unicode.",
                checked_args.csl
            ))?
        };

        // Create bibliography as a string
        let mut bibliography = String::default();
        let html_template = if checked_args.nohtml {
            |counter: usize, content: String| format!("[{}] {}\n", counter, content)
        } else {
            |counter: usize, content: String| {
                format!(
                    "<p>[<a id=\"fn:{}\">{}</a>] {}</p>\n",
                    counter, counter, content
                )
            }
        };
        for (mut i, cite) in result.bibliography.unwrap().items.iter().enumerate() {
            i += 1;
            let mut cite_buffer: String = String::new();
            cite.content
                .write_buf(&mut cite_buffer, hayagriva::BufWriteFormat::Plain)
                .unwrap();
            bibliography = bibliography + &html_template(i, cite_buffer);
        }

        // Write bibliography string to file (if not a dry run or writing to term)
        let markdown_with_bib = edited_markdown.to_string() + "\n" + &bibliography;
        if checked_args.dryrun {
            println!("{} ", "[Raw Bibliography]".green());
            println!("{:?}", bibliography);
        } else if checked_args.term {
            println!("{}", bibliography);
        } else {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(checked_args.output)?;
            file.write_all(markdown_with_bib.as_bytes()).unwrap();
        }
    } else {
        Err(anyhow!("Error reading the contents of the file."))?;
    };

    Ok(())
}
