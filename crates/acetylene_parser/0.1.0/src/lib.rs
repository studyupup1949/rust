//! A string parser for different chemical nomenclature. Focused on SMILES.

extern crate regex;
#[macro_use] extern crate lazy_static;

mod formula;
mod smiles;

pub mod types;

use formula::tokenize_formula;
use smiles::tokenize_smiles;
use types::Substance;

/// Tokenizes a string describing a chemical, yielding a Substance with
/// (optional) functional groups corresponding to (more) fundamental components.
///
/// "kind" can be one of two strings: "formula" or "smiles". "formula" expects a
/// simple molecular formula notation. "smiles" expects an OpenSMILES-specified
/// string. See [the tutorial at Daylight][ref] for info.
///
/// [ref]: http://www.daylight.com/dayhtml_tutorials/languages/smiles/index.html
///
/// # Examples
///
/// ## Molecular formula parsing
///
/// Here are a couple examples of what molecular formula parsing can do.
///
/// Parsing H2SO4 (sulfuric acid) without any structural annotations:
///
/// ```
/// 
/// use acetylene_parser::tokenize;
/// use acetylene_parser::types::Substance;
///
/// assert_eq!(
///   tokenize("H2SO4", "formula"),
///   Substance {
///     symbol: String::from("H2SO4"),
///     quantity: 1,
///     charge: None,
///     groups: Some(Box::new(vec![
///       Substance { symbol: String::from("H"), quantity: 2, charge: None, groups: None },
///       Substance { symbol: String::from("S"), quantity: 1, charge: None, groups: None },
///       Substance { symbol: String::from("O"), quantity: 4, charge: None, groups: None }
///     ]))
///   }
/// );
/// ```
///
/// In this case, the parser just pulls out the individual elements and tallies
/// them up.
///
/// If we annotate the structure with parentheses (denoting groups), we'll get
/// something slightly different:
///
/// 
///
/// ```
/// 
/// use acetylene_parser::tokenize;
/// use acetylene_parser::types::Substance;
///
/// assert_eq!(
///   tokenize("H2(SO4)", "formula"),
///   Substance {
///     symbol: String::from("H2(SO4)"),
///     quantity: 1,
///     charge: None,
///     groups: Some(Box::new(vec![
///       Substance { symbol: String::from("H"), quantity: 2, charge: None, groups: None },
///       Substance { symbol: String::from("(SO4)"), quantity: 1, charge: None, groups: None },
///       Substance { symbol: String::from("S"), quantity: 1, charge: None, groups: None },
///       Substance { symbol: String::from("O"), quantity: 4, charge: None, groups: None },
///     ]))
///   }
/// );
/// ```
///
/// This time we can explicitly parse out both dihydrogen (H2) and sulfate
/// (SO4). Note that charges are not inferred.
///
/// There's not much error checking going on in either case--only regex for now.
///
/// ## SMILES parsing
///
/// SMILES parsing has access to much more elaborate structural information. For
/// example, ring closures are annotations that allow a ring of molecules to be
/// unrolled into a 1-dimensional string, by marking the two breaks with a
/// number following the atoms that need to reconnect.
///
/// For example, benzene looks like this:
///
/// ```
/// 
/// use acetylene_parser::tokenize;
/// use acetylene_parser::types::Substance;
///
/// let test_str = r"c1ccccc1";
/// let res = tokenize(test_str, "smiles");
/// 
/// let sub = Substance {
///   symbol: String::from("c1ccccc1"),
///   quantity: 1,
///   charge: None,
///   groups: Some(Box::new(vec![
///     Substance {
///       symbol: String::from("c1ccccc1"),
///       quantity: 1,
///       charge: None,
///       groups: None
///     },
///     Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
///     Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
///     Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
///     Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
///     Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
///     Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None }      
///   ]))
/// };
///
/// assert_eq!(res, sub);
/// ```
///
/// The first functional group listed is identical to the "whole substance"
/// described with the input string, which makes sense--the whole string is
/// composed of one big ring closure, and so the parser detects that and rolls
/// it all up.
///
/// Note that the individual atoms in the ring aren't collected up into a single
/// molecular formula. I'm still working out how I want to go about it.
///
/// Here's one last example with hydronium. Hydronium is an ion, so it's
/// required to be in brackets.
/// 
/// ```
/// 
/// use acetylene_parser::tokenize;
/// use acetylene_parser::types::Substance;
///
/// let test_str = r"[OH3+]";
/// let res = tokenize(test_str, "smiles");
///
/// let sub = Substance {
///   symbol: String::from("[OH3+]"),
///   quantity: 1,
///   charge: Some(1),
///   groups: Some(Box::new(vec![
///     Substance {
///       symbol: String::from("[OH3+]"),
///       quantity: 1,
///       charge: Some(1),
///       groups: None
///     }
///   ]))
/// };
///
/// assert_eq!(res, sub);
/// ```
///
pub fn tokenize(input: &str, kind: &str) -> Substance {
  match kind {
    "formula" => tokenize_formula(input),
    "smiles" => tokenize_smiles(input),
//    "iupac" => tokenize_iupac(input),
    _ => tokenize_formula(input)
  }
}
