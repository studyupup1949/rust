use absurd::append_after_marker;

#[test]
fn can_append_text_alphabetically() {
    let text = r#"
        pub mod bar;
        pub mod foo;
    "#.to_string();
    let new_line = "        pub mod baz;".to_string();
    dbg!(&text);
    let text = append_after_marker(text, "        pub mod", new_line);
    dbg!(&text);
    assert_eq!(&text, r#"
        pub mod bar;
        pub mod baz;
        pub mod foo;
    "#);
}
