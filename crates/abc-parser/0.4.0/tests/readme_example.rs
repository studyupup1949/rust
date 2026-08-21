use pretty_assertions::assert_eq;

#[test]
fn readme_example() {
    use abc_parser::abc;
    use abc_parser::datatypes::*;

    let parsed = abc::tune_book("X:1\nT:Example\nK:D\n").unwrap();
    assert_eq!(
        parsed,
        TuneBook::new(
            vec![],
            None,
            vec![Tune::new(
                TuneHeader::new(vec![
                    HeaderLine::Field(InfoField::new('X', "1".to_string()), None),
                    HeaderLine::Field(InfoField::new('T', "Example".to_string()), None),
                    HeaderLine::Field(InfoField::new('K', "D".to_string()), None)
                ]),
                None
            )]
        )
    )
}
