extern crate abc_parser;
use abc_parser::abc;
use abc_parser::datatypes::writer::ToABC;

#[test]
fn to_ast_and_back_again() {
    let data = "M:4/4
O:Irish
R:Reel

X:1
T:Untitled Reel
C:Trad.
K:D
eg|a2ab ageg|agbg agef|g2g2 fgag|f2d2 d2:|
ed|cecA B2ed|cAcA E2ed|cecA B2ed|c2A2 A2:|
AB|cdec BcdB|ABAF GFE2|cdec BcdB|c2A2 A2:|
";
    let tb = abc::tune_book(data).unwrap();
    let formatted = tb.to_abc();
    println!("\n{}", formatted);
    assert_eq!(data, formatted);
    abc::tune_book(&formatted).unwrap();
}
