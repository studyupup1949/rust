use crate::errors::*;
use chumsky::prelude::*;
use std::fs;
use std::collections::HashMap;

// Error handling
type Result<T> = std::result::Result<T, AcfError>;

/// Representation of an ACF's file content
/// 
/// Results are returned in the form of a hash map. Valve ACF files are expected
/// to have a root level entry (`AppState`) containing the app's ID, path, name,
/// and filesystem specific information
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Acf {
    /// A list of entries. Valve ACF files should have at least `AppState`
    pub entries: Vec<Entry>,
}

/// Representation of an individual ACF entry
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Entry {
    /// Name of the entry
    pub name: String,

    // A list of expressions
    pub expressions: HashMap<String, String>,

    // A list of sub-entries
    pub entries: Vec<Entry>,
}

/// Representation of an individual ACF expression (of form "*."\s+"*.")
/// 
/// > NOTE: This is an internal representation that is not shown to the user
#[derive(Clone, Debug, PartialEq, Eq)]
struct Expr {
    /// Name of the expression
    name: String,

    /// Value of the expression
    value: String,
}

/// ACF file parser
///
/// An ACF file is just a list of ACF entries. The current implementation returns a vector of
/// entries, but expects a single root entry. It will not parse files that have additional entries given
pub fn parse_acf(path: &str) -> Result<Acf> {
    let contents = match fs::read_to_string(path) {
        Ok(val) => val,
        Err(_) => return Err(AcfError::Read(path.into())),
    };

    let entries = match acf_parser().parse(&contents).into_result() {
        Ok(val) => val,
        Err(e) => {
            e.into_iter()
                .for_each(|err| println!("Parse error: {}", err));
            return Err(AcfError::Parse(ParseError::Unknown));
        }
    };

    Ok(Acf { entries })
}

/// ACF parser
///
/// A wrapper for the entry parser that allows multiple entries to be defined within the file.
/// Will parse until the end of the file is reached
fn acf_parser<'src>() -> impl Parser<'src, &'src str, Vec<Entry>> {
    entry_parser()
        .padded()
        .repeated()
        .collect::<Vec<_>>()
        .then_ignore(end())
        .map(|entries| entries)
}

/// Entry parser
///
/// Entries start with a string literal followed by an opening brace (i.e., '{'). Entries are
/// expected to have a list of expressions, followed by a list of sub-entries. This ordering
/// is currently enforced
fn entry_parser<'src>() -> impl Parser<'src, &'src str, Entry> {
    recursive(|rec_parser| {
        str_parser()
            .padded()
            .then_ignore(just("{").padded())
            .then(
                expr_parser().padded().repeated().collect::<Vec<_>>()
            )
            .then(rec_parser.padded().repeated().collect::<Vec<_>>())
            .then_ignore(just("}").padded())
            .map(|((name, expressions), entries)| Entry {
                name,
                expressions: {
                    let names = expressions.iter().map(|expr| expr.name.clone());
                    let values = expressions.iter().map(|expr| expr.value.clone());

                    names.zip(values).collect()
                },
                entries,
            })
            .boxed()
    })
}

/// Expression parser
///
/// Expressions are formed by two string literals delimited by some whitespace. There are no
/// constraints as to what may form entries (will match up until next quote), so you may get
/// strange resulting expressions if the input file is incorrectly formatted
fn expr_parser<'src>() -> impl Parser<'src, &'src str, Expr> {
    str_parser()
        .padded()
        .then(str_parser())
        .padded()
        .map(|(str1, str2)| Expr {
            name: str1,
            value: str2,
        })
}

/// String literal parser
fn str_parser<'src>() -> impl Parser<'src, &'src str, String> {
    just('"')
        .ignore_then(none_of('"').repeated().to_slice())
        .then_ignore(just('"'))
        .padded()
        .map(|val: &str| val.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_run() {
        let result = parse_acf("./acfs/simple.acf");
        assert!(result.is_ok());
    }

    #[test]
    fn simple() {
        let result = parse_acf("./acfs/simple.acf");
        assert!(result.is_ok());
        let result = result.unwrap();
        let root_entry = &result.entries[0];
        assert_eq!(root_entry.name, "AppState");
        let expressions = &root_entry.expressions;
        assert_eq!(expressions["appid"], "730");
    }

    #[test]
    fn full() {
        let result = parse_acf("./acfs/appmanifest_730.acf");
        assert!(result.is_ok());
        let result = result.unwrap();
        let root_entry = &result.entries[0];
        assert_eq!(root_entry.name, "AppState");
        let expressions = &root_entry.expressions;
        assert_eq!(expressions["appid"], "730");
        assert_eq!(expressions["universe"], "1");
        assert_eq!(expressions["LauncherPath"], "C:\\\\Program Files (x86)\\\\Steam\\\\steam.exe");
        assert_eq!(expressions["name"], "Counter-Strike 2");
    }
}
