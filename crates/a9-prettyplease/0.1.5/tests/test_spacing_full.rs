#[test]
fn test_spacing_full() {
    let code = std::fs::read_to_string("examples/input.rs").unwrap();
    let syntax_tree = syn::parse_file(&code).unwrap();
    let formatted = a9_prettyplease::unparse(&syntax_tree);
    std::fs::write("examples/output.spacing.rs", &formatted).unwrap();
    print!("{}", &formatted[..2000.min(formatted.len())]);
}
