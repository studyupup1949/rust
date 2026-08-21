extern crate acetylene_parser;

use acetylene_parser::types::Substance;
use acetylene_parser::tokenize;

#[test]
fn tokenize_formula_simple() {
  let sub = Substance {
    symbol: "H".to_string(),
    quantity: 1,
    charge: None,
    groups: None
  };

  assert_eq!(tokenize("H", "formula"),
             sub);
}

#[test]
fn tokenize_formula_nested() {
  // isobutane with structural notation (parens)
  let sub = Substance {
    symbol: "(CH3)3CH".to_string(),
    quantity: 1,
    charge: None,
    groups: Some(Box::new(vec![
      Substance {
        symbol: "(CH3)".to_string(), // methyl group
        quantity: 3,
        charge: None,
        // NOTE technically CH3 is an ion, but the formula string doesn't
        // express that
        groups: None
      },
      Substance {
        symbol: "C".to_string(),
        quantity: 1,
        charge: None,
        groups: None
      },
      Substance {
        symbol: "H".to_string(),
        quantity: 3,
        charge: None,
        groups: None
      }
    ]))
  };

  assert_eq!(tokenize("(CH3)3CH", "formula"),
             sub);
}

#[test]
fn tokenize_benzene_smiles() {
  let test_str = r"c1ccccc1";
  let res = tokenize(test_str, "smiles");
  
  let sub = Substance {
    symbol: String::from("c1ccccc1"),
    quantity: 1,
    charge: None,
    groups: Some(Box::new(vec![
      Substance {
        symbol: String::from("c1ccccc1"),
        quantity: 1,
        charge: None,
        groups: None
      },
      Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
      Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
      Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
      Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
      Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None },
      Substance { symbol: String::from("c"), quantity: 1, charge: None, groups: None }      
    ]))
  };

  assert_eq!(res, sub);
}

#[test]
fn tokenize_hydronium_smiles() {
  let test_str = r"[OH3+]";
  let res = tokenize(test_str, "smiles");

  let sub = Substance {
    symbol: String::from("[OH3+]"),
    quantity: 1,
    charge: Some(1),
    groups: Some(Box::new(vec![
      Substance {
        symbol: String::from("[OH3+]"),
        quantity: 1,
        charge: Some(1),
        groups: None
      }
    ]))
  };

  assert_eq!(res, sub);
}

#[test]
#[ignore]
fn tokenize_iupac() {
  
}

#[test]
#[ignore]
fn tokenize_titin() {
  
}
