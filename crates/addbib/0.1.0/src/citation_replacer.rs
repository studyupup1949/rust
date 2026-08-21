/*
* Copyright (c) 2025 Andrew Rowan Barlow. Licensed under the EUPL-1.2
* or later. You may obtain a copy of the licence at
* https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12. A copy
* of the EUPL-1.2 licence in English is given in LICENSE.txt which is
* found in the root directory of this repository.
*
* Author: Andrew Rowan Barlow <a.barlow.dev@gmail.com>
*/

use crate::without_first;
use colored::Colorize;
use regex::Captures;
use std::collections::HashMap;

fn html_citation(count: usize) -> String {
    format!("<a href=\"#fn:{count}\" class=\"footnote-ref\" role=\"doc-noteref\">{count}</a>")
}

fn html_single_citation(count: usize) -> String {
    format!("[{}]", html_citation(count))
}

/// An adapted regex::Replacer that record the citations and associated number.
pub struct CitationReplacer {
    pub citations: HashMap<String, usize>,
    pub counter: usize,
    pub dryrun: bool,
    pub nohtml: bool,
}

impl CitationReplacer {
    pub fn new(dryrun: bool, nohtml: bool) -> Self {
        CitationReplacer {
            citations: HashMap::default(),
            counter: 1,
            dryrun,
            nohtml,
        }
    }
}

impl regex::Replacer for &mut CitationReplacer {
    /// Function for replacing markdown citations, as defined in README.md,
    /// with numbers, e.g. [1].
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        let html_single_template = if self.nohtml {
            |counter: usize| format!("[{}]", counter)
        } else {
            |counter: usize| html_single_citation(counter)
        };
        let html_long_template = if self.nohtml {
            |counter: usize| counter.to_string()
        } else {
            |counter: usize| html_citation(counter)
        };

        if let Some(matched) = caps.get(2) {
            // @Author2025 is matched
            let matched_str: String = matched.as_str().to_owned();
            let html_string = if let Some(&index) = self.citations.get(&matched_str) {
                html_single_template(index)
            } else {
                self.citations.insert(matched_str, self.counter);
                self.counter += 1;
                html_single_template(self.counter - 1usize)
            };
            if self.dryrun {
                println!(
                    "{}{} {} {:?} -> {}",
                    "[Citation found : ".green(),
                    matched.start().to_string().green(),
                    "]".green(),
                    matched.as_str(),
                    html_string
                )
            }
            dst.push_str(&html_string);
        } else {
            // [@AuthorOne2025] or [@AuthorOne2025, AuthorTwo2025, ...] is matched
            let matched = caps.get(0).unwrap(); //guaranteed to be Some
            let mut citations_list = matched.as_str().to_string();
            citations_list.remove(0); // removes [
            citations_list.pop(); // removes ]
            let md_citations = citations_list.split(",").collect::<Vec<&str>>();
            let mut html_citations: Vec<String> = Vec::default();
            for cite in md_citations {
                let cleaned_cite = without_first(cite.trim());
                if let Some(&index) = self.citations.get(cleaned_cite) {
                    html_citations.push(html_long_template(index));
                } else {
                    self.citations
                        .insert(cleaned_cite.to_string(), self.counter);
                    self.counter += 1;
                    html_citations.push(html_long_template(self.counter - 1usize));
                }
            }
            let mut swapped_citations = html_citations
                .iter()
                .fold("".to_string(), |acc, x| acc.to_owned() + x + ", ");
            swapped_citations.pop();
            swapped_citations.pop(); //removes ", " from end
            dst.push_str(&format!("[{swapped_citations}]"));
            if self.dryrun {
                println!(
                    "{}{} {} {:?} -> {}",
                    "[Citation found : ".green(),
                    matched.start().to_string().green(),
                    "]".green(),
                    matched.as_str(),
                    &format!("[{swapped_citations}]")
                )
            }
        }
    }
}
