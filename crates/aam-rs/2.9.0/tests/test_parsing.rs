#[cfg(test)]
mod tests {
    use aam_rs::aam::AAM;
    use aam_rs::builder::{AAMBuilder, SchemaField};
    use aam_rs::found_value::FoundValue;
    use std::fs;

    #[test]
    fn color_not_treated_as_comment() {
        let doc = AAM::parse("tint = #ff6600").expect("parse should succeed");
        assert_eq!(doc.get("tint"), Some("#ff6600"));
    }

    #[test]
    fn comment_after_space_is_ignored() {
        let doc = AAM::parse("key = value # comment").expect("parse should succeed");
        assert_eq!(doc.get("key"), Some("value"));
    }

    #[test]
    fn quoted_hash_is_preserved() {
        let doc = AAM::parse(r#"key = "val # not comment""#).expect("parse should succeed");
        assert_eq!(doc.get("key"), Some("\"val # not comment\""));
    }

    #[test]
    fn inline_object_and_list_values_parse() {
        let content = "obj = { x = 1, y = 2 }\nitems = [a, b, c]";
        let doc = AAM::parse(content).expect("parse should succeed");

        assert_eq!(doc.get("obj"), Some("{ x = 1, y = 2 }"));
        assert_eq!(doc.get("items"), Some("[a, b, c]"));
    }

    #[test]
    fn comments_and_not_comments() {
        let content = r#"
        # comments
        obj = #NotComment # Comment
        #comment = #NotComment # Comment
        "#;
        let doc = AAM::parse(content).expect("parse should succeed");
        assert_eq!(doc.get("obj"), Some("#NotComment"));
        assert_eq!(doc.get("#comment"), Some("#NotComment"));
    }

    #[test]
    fn incompability_type() {
        let content = r#"
        @schema Ok {
            a: i32,
            b: u32
        }
        "#;
        let doc = AAM::parse(content);
        assert!(doc.is_err());
    }

    #[test]
    fn schema_not_valid() {
        let content = r#"
        @schema NotValid {
            a: i32
            b: # Ok
            c: #OK
            d: \n
            d: f32
        }
        "#;
        let doc = AAM::parse(content);
        assert!(doc.is_err());
    }

    #[test]
    fn schema_in_schema_in_schema() {
        let content = r#"@schema Ok {
            a: i32
            b: i32
        }
        @schema Second {
            c: Ok
            d: i32
        }
        @schema Third {
            e: Second
            f: Ok
        }

        let third = {{{1, 2}, 3}, {4, 5}}
        "#;
        let doc = AAM::parse(content).expect("parse should succeed");
        assert_eq!(doc.get("let third"), Some("{{{1, 2}, 3}, {4, 5}}"));
    }

    #[test]
    fn type_in_type() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let base_path = dir.path();

        let file_path = base_path.join("type_in_type.aam");
        let child_path = base_path.join("type_in_type_child.aam");

        let mut builder = AAMBuilder::new();
        builder.type_alias("A", "i32");
        builder.type_alias("B", "A");
        builder.type_alias("C", "list<B>");
        builder.type_alias("D", "list<C>");
        builder.schema_multiline(
            "Ok",
            [
                SchemaField::required("field1", "D"),
                SchemaField::required("field2", "C"),
            ],
        );

        builder
            .to_file(&file_path)
            .expect("Failed to write AAM file");

        let mut builder = AAMBuilder::new();
        builder.add_line("field1", "[[1, 2], [3, 4]]");
        builder.add_line("field2", "[1, 2, 3, 4]");

        builder
            .to_file(&child_path)
            .expect("Failed to write AAM file");

        let doc = AAM::load(&child_path).expect("Failed to load AAM file");

        assert_eq!(doc.get("field1"), Some("[[1, 2], [3, 4]]"));
        assert_eq!(doc.get("field2"), Some("[1, 2, 3, 4]"));
    }

    #[test]
    fn schema_matryoshka_inception() {
        let content = r#"
        @schema Lvl1 { val: i32 }
        @schema Lvl2 { l1: Lvl1 }
        @schema Lvl3 { l2: Lvl2 }
        @schema Lvl4 { l3: Lvl3 }
        @schema Boss { final: Lvl4 }

        boss_fight = {{{{1}}}}
        "#;
        let doc = AAM::parse(content).expect("Inception-schema must be parsed");
        assert_eq!(doc.get("boss_fight"), Some("{{{{1}}}}"));
    }

    #[test]
    fn cursed_comments_everywhere() {
        let content = r#"
        # Погнали
        key1 = value1 # comment
        key2 = #not_a_comment # This is a comment, but the value is #not_a_comment
        list = [1, 2, 3] # Comment after list

        # Just a comment
        # Just a comment
        # Just a comment
        # Just a comment
        # Just a comment
        # Just a comment
        # Just a comment
        # Just a comment
        # Just a comment
        # Just a comment
        # Just a comment# Just a comment# Just a comment# Just a comment# Just a comment# Just a comment# Just a comment# Just a comment
        obj = { x = 1, y = 2 } # Comment # after # Comment
        "#;
        let doc = AAM::parse(content).expect("Should be success with all these comments");
        assert_eq!(doc.get("key1"), Some("value1"));
        assert_eq!(doc.get("key2"), Some("#not_a_comment"));
        assert_eq!(doc.get("list"), Some("[1, 2, 3]"));
        assert_eq!(doc.get("obj"), Some("{ x = 1, y = 2 }"));
    }

    #[test]
    fn schema_with_invalid_comments_and_missing_types() {
        let content = r#"
        @schema BrokenUser {
            name: string # Here it's ok
            age: # User is using Ai, you should know here must be a type
            score: f64
        }
        "#;
        let doc = AAM::parse(content);
        assert!(doc.is_err());
    }

    #[test]
    fn unclosed_brackets_should_panic() {
        let content_list = "bad_list = [1, 2, 3 \n next_key = 4";
        let doc1 = AAM::parse(content_list);
        assert!(doc1.is_err());

        let content_obj = "bad_obj = { x = 1, y = 2 \n";
        let doc2 = AAM::parse(content_obj);
        assert!(doc2.is_err());
    }

    #[test]
    fn whitespace_chaos() {
        let content =
            "spaced_out   =    \t  value_here   \n\n\n another_key = \t [  1 ,   2 ] \n\n";
        let doc = AAM::parse(content).expect("Tabs and multiple spaces should be handled");

        assert_eq!(doc.get("spaced_out"), Some("value_here"));
        assert_eq!(doc.get("another_key"), Some("[1, 2]"));
    }

    #[test]
    fn empty_values_and_keys_are_invalid() {
        let content_no_val = "empty_value_key = \n";
        let doc1 = AAM::parse(content_no_val);
        assert!(doc1.is_err(), "Key without a value must be error");

        let content_no_key = "= just_value \n";
        let doc2 = AAM::parse(content_no_key);
        assert!(doc2.is_err());
    }

    #[test]
    fn edge_case_strings_with_equals_and_hashes() {
        let content = r#"
        complex_string = "key = value # this is NOT a comment"
        "#;
        let doc = AAM::parse(content).expect("Parsing Should succeed");
        assert_eq!(
            doc.get("complex_string"),
            Some("\"key = value # this is NOT a comment\"")
        );
    }

    #[test]
    fn type_recursively() {
        let content = r#"
        @type A = B
        @type C = A
        @type B = C
        @type A = i32
        "#;
        let doc = AAM::parse(content);
        match doc {
            Ok(data) => println!("Успех: {:?}", data),
            Err(errors) => {
                for err in errors {
                    println!("{}", err);
                }
            }
        }
    }

    #[test]
    fn test_multiline() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let base_path = dir.path();

        let file_path = base_path.join("multiline.aam");
        let child_path = base_path.join("multiline_child.aam");

        let mut b = AAMBuilder::new();
        b.schema_multiline(
            "Multiline",
            vec![
                SchemaField::required("id", "i32"),
                SchemaField::required("name", "string"),
                SchemaField::required("deps", "list<string>"),
            ],
        )
        .schema_multiline(
            "Base",
            vec![SchemaField::required("multiline", "Multiline")],
        )
        .to_file(&file_path)
        .expect("Failed to write AAM file");
        let content = r#"
        multiline = {
            id = 32,
            name = Usein,
            deps = [
                "a",
                "b",
                "c"
            ]
        }
        "#;
        fs::write(&child_path, content).expect("Failed to write child AAM file");
        let doc = AAM::load(child_path).expect("Should succeed");
        let parsed_doc = FoundValue::new(doc.get("multiline").expect("multiline key should exist"))
            .as_object()
            .expect("multiline value should be an object");
        let parsed_deps =
            FoundValue::new(parsed_doc.get("deps").expect("Multiline deps should exist"))
                .parse_list::<String>()
                .expect("deps should be a list")
                .expect("deps should be parsed successfully");
        assert_eq!(parsed_deps, vec!["a", "b", "c"]);
    }

    #[test]
    fn quoted_list_items_with_commas_inside() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let base_path = dir.path();

        let file_path = base_path.join("quoted_list.aam");
        let child_path = base_path.join("quoted_list_child.aam");

        let mut b = AAMBuilder::new();
        b.schema_multiline(
            "Post",
            vec![
                SchemaField::required("id", "i32"),
                SchemaField::required("tags", "list<string>"),
            ],
        )
        .to_file(&file_path)
        .expect("Failed to write AAM file");

        let content = r#"
        id = 1
        tags = ["hello, world", "rust, aam", plain]
        "#;
        fs::write(&child_path, content).expect("Failed to write child AAM file");
        let doc = AAM::load(child_path).expect("Should succeed");
        let tags = FoundValue::new(doc.get("tags").expect("tags should exist"))
            .parse_list::<String>()
            .expect("tags should be a list")
            .expect("tags should be parsed successfully");
        assert_eq!(tags, vec!["hello, world", "rust, aam", "plain"]);
    }

    #[test]
    fn quoted_list_items_single_quotes() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let base_path = dir.path();

        let file_path = base_path.join("single_quote_list.aam");
        let child_path = base_path.join("single_quote_list_child.aam");

        let mut b = AAMBuilder::new();
        b.schema_multiline("Data", vec![SchemaField::required("items", "list<string>")])
            .to_file(&file_path)
            .expect("Failed to write AAM file");

        let content = r#"
        items = ['a, b', 'c, d', plain]
        "#;
        fs::write(&child_path, content).expect("Failed to write child AAM file");
        let doc = AAM::load(child_path).expect("Should succeed");
        let items = FoundValue::new(doc.get("items").expect("items should exist"))
            .parse_list::<String>()
            .expect("items should be a list")
            .expect("items should be parsed successfully");
        assert_eq!(items, vec!["a, b", "c, d", "plain"]);
    }

    #[test]
    fn quoted_list_items_no_schema() {
        let content = r#"tags = ["rust, aam", "cli, tool", simple]"#;
        let doc = AAM::parse(content).expect("parse should succeed");
        assert_eq!(
            doc.get("tags"),
            Some("[\"rust, aam\", \"cli, tool\", simple]")
        );
    }
}
