/*
* Copyright (c) 2025 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENSE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::anyhow;
use crate::Args;
use hayagriva::citationberg::{IndependentStyle, Locale, LocaleFile};
use std::fs;
use std::path::PathBuf;

pub struct CheckedArgs {
    pub bib: PathBuf,
    pub csl: PathBuf,
    pub output: PathBuf,
    pub dryrun: bool,
    pub strict: bool,
    pub nohtml: bool,
    pub term: bool,
    pub markdown: PathBuf,
}

pub(crate) fn check_args(args: Args) -> anyhow::Result<CheckedArgs> {
    check_file(&args.markdown)?;
    let output_file = args.output.unwrap_or(args.markdown.clone());
    check_file(&args.csl)?;
    check_file(&args.bib)?;
    Ok(CheckedArgs {
        bib: args.bib,
        csl: args.csl,
        output: output_file,
        dryrun: args.dryrun,
        strict: args.strict,
        nohtml: args.nohtml,
        term: args.term,
        markdown: args.markdown,
    })
}

pub fn without_first(string: &str) -> &str {
    string
        .char_indices()
        .nth(1)
        .and_then(|(i, _)| string.get(i..))
        .unwrap_or("")
}

fn check_file(file: &PathBuf) -> anyhow::Result<()> {
    if !file.exists() {
        return Err(anyhow!("File, {:?}, does not exist.", file));
    }
    if !file.is_file() {
        return Err(anyhow!(
            "The selected output file, {:?}, is not a file.",
            file
        ));
    }
    Ok(())
}

pub fn setup_locale_styles(
    locale_str: &str,
    style_path: &str,
) -> Result<([Locale; 1], IndependentStyle), anyhow::Error> {
    let locales = [LocaleFile::from_xml(locale_str).unwrap().into()];
    let style = fs::read_to_string(style_path).map_err(|_| {
        anyhow!(
            "Could not find citation style file {:?}",
            style_path.to_owned()
        )
    })?;
    let style = IndependentStyle::from_xml(&style).unwrap();
    Ok((locales, style))
}

pub trait MoveOption<T>
where
    T: Clone,
{
    fn move_out(self, inplace: &T) -> T;
}

impl<T> MoveOption<T> for Option<T>
where
    T: Clone,
{
    fn move_out(mut self, inplace: &T) -> T {
        if let Some(value) = self.take() {
            value
        } else {
            inplace.clone()
        }
    }
}
