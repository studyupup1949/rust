use clap::{Parser, Subcommand};

use crate::model::{Language, CStandard, CppStandard};

#[derive(Parser, Debug)]
pub struct AcmakeArgs {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    New {
        name: String,

        #[arg(short, long, value_parser=parse_language)]
        language: Language,

        #[arg(short, long)]
        folder: Option<String>,
    },
    Init {
        name: String,

        #[arg(short, long, value_parser=parse_language)]
        language: Language,
    },
}

fn parse_language(lang: &str) -> Result<Language, String> {
    let input: Vec<&str> = lang.split(':').collect();
    let lang = input[0];
    let standard = input.get(1);
    let supported_langs = ["c", "cpp", "cxx", "c++"];

    if supported_langs[0] == lang {
        let standard = standard.map(|t| *t).unwrap_or("11");
        let standard = CStandard::try_from(standard)?;
        return Ok(Language::C(standard));
    }

    if supported_langs[1..].contains(&lang) {
        let standard = standard.map(|t| *t).unwrap_or("17");
        let standard = CppStandard::try_from(standard)?;
        return Ok(Language::CPP(standard));
    }

    let formatted_possible_values = supported_langs.join(", ");
    Err(format!(
        "Possible values are {:?}",
        formatted_possible_values
    ))
}
