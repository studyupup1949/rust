use std::env::{self, Args};
use std::error::Error;
use std::fs;
pub struct Config {
    pub file_name: String,
    pub query: String,
    pub case_sensitiv: bool,
}
impl Config {
    pub fn new(mut args: Args) -> Result<Config, &'static str> {
        args.next();
        let query = match args.next() {
            Some(q) => q,
            None => return Err("didn't get a query string"),
        };
        let file_name = match args.next() {
            Some(q) => q,
            None => return Err("didn't get file name  string"),
        };
        let case_sensitiv = env::var("CASE_INSENSITIVE").is_err();
        Ok(Config {
            file_name: file_name,
            query: query,
            case_sensitiv: case_sensitiv,
        })
    }
}
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(config.file_name)?;
    let result = if config.case_sensitiv {
        search(&config.query, &content)
    } else {
        search_case_insensitive(&config.query, &content)
    };

    for line in result {
        println!("{}", line);
    }
    Ok(())
}

pub fn search<'a>(query: &str, content: &'a str) -> Vec<&'a str> {
    content.lines().filter(|str| str.contains(query)).collect()
}
pub fn search_case_insensitive<'a>(query: &str, content: &'a str) -> Vec<&'a str> {
    content
        .lines()
        .filter(|str| str.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let content = "/ Rust:
safe, fast, productive
.
Pick three.";
        assert_eq!(vec!["safe, fast, productive"], search(query, content));
    }
    #[test]
    fn case_sensitive() {
        let query = "rUsT";
        let content = "/
Rust:
safe, fast, productive
.
Pick three.
trust me.";
        assert_eq!(
            vec!["Rust:", "trust me."],
            search_case_insensitive(query, content)
        );
    }
}
