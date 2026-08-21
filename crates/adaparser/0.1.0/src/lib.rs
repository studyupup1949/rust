use lalrpop_util::lalrpop_mod;
use aleph_syntax_tree::syntax::AlephTree as at;

lalrpop_mod!(pub grammar); // Generated parser from grammar.lalrpop

/// Parses an Ada program into an AlephTree.
///
/// # Arguments
/// * `source` - String containing the Ada code to parse.
///
/// # Returns
/// An `AlephTree` representing the parsed Ada program.
/// In case of error, returns `at::Unit` and prints an error message.
pub fn parse(source: String) -> at {
    let ast = grammar::ProgramParser::new().parse(&source);
    match ast {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            at::Unit
        }
    }
}

/// Parses a list of Ada declarations.
///
/// # Arguments
/// * `source` - String containing the Ada declarations.
///
/// # Returns
/// A `Vec<Box<at>>` of parsed declarations.
/// In case of error, returns an empty vector and prints an error message.
pub fn parse_ada_declarations(source: String) -> Vec<Box<at>> {
    let ast = grammar::DeclarationListParser::new().parse(&source);
    match ast {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            Vec::new()
        }
    }
}

/// Parses a list of Ada statements.
///
/// # Arguments
/// * `source` - String containing the Ada statements.
///
/// # Returns
/// A `Vec<Box<at>>` of parsed statements.
/// In case of error, returns an empty vector and prints an error message.
pub fn parse_ada_statements(source: String) -> Vec<Box<at>> {
    let ast = grammar::StatementListParser::new().parse(&source);
    match ast {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            Vec::new()
        }
    }
}

