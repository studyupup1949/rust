#[cfg(test)]
mod tests {
    use crate::aaml::AAML;
    use crate::found_value::FoundValue;

    const TEST_CONFIG: &str = "
        a = b
        c = d
        e = f
        d = g
        loop1 = loop2
        loop2 = loop1
    ";

    #[test]
    fn test_simple_find() {
        let parser = AAML::parse(TEST_CONFIG).expect("Error parsing config");
        let res = parser.find_obj("a").expect("Should find 'a'");
        assert_eq!(res, "b");
    }

    #[test]
    fn test_not_found() {
        let parser = AAML::parse(TEST_CONFIG).expect("Error parsing config");
        assert!(parser.find_obj("unknown").is_none());
    }

    #[test]
    fn test_deref_behavior() {
        let parser = AAML::parse(TEST_CONFIG).expect("Error parsing config");
        let res = parser.find_obj("a").unwrap();

        assert_eq!(res.len(), 1);
        assert!(res.starts_with('b'));
    }

    #[test]
    fn test_display_trait() {
        let res = FoundValue::new(&*"hello".to_string());
        let formatted = format!("{}", res);
        assert_eq!(formatted, "hello");
    }

    #[test]
    fn test_find_deep() {
        let parser = AAML::parse(TEST_CONFIG).expect("Error parsing config");
        let res = parser.find_deep("c").expect("Should find 'c'");
        assert_eq!(res, "g");
    }

    #[test]
    fn test_find_deep_direct_loop() {
        let content = "key1=key1";
        let aaml = AAML::parse(content).expect("Error parsing config");

        let result = aaml.find_deep("key1");
        assert_eq!(result.unwrap().as_str(), "key1");
    }

    #[test]
    fn test_find_deep_indirect_loop() {
        let content = "a=b\nb=a";
        let aaml = AAML::parse(content).expect("Error parsing config");

        let result = aaml.find_deep("a");
        assert_eq!(result.unwrap().as_str(), "b");
    }

    #[test]
    fn test_find_deep_long_chain_with_loop() {
        let content = "start=mid\nmid=end\nend=mid";
        let aaml = AAML::parse(content).expect("Error parsing config");

        let result = aaml.find_deep("start");
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "end");
    }

    #[test]
    fn test_find_deep_no_loop() {
        let content = "a=b\nb=c\nc=final";
        let aaml = AAML::parse(content).expect("Error parsing config");

        let result = aaml.find_deep("a");
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "final");
    }

    #[test]
    fn test_parse_error_missing_equals() {
        let content = "invalid_line_without_equals";
        let res = AAML::parse(content);
        assert!(res.is_err());
        let err = res.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Missing assignment operator"));
    }

    #[test]
    fn test_parse_error_empty_key() {
        let content = "= value";
        let res = AAML::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn test_quotes_stripping() {
        let content = r#"
            key1 = "value1"
            key2 = 'value2'
            key3 = value3
            key4 =
        "#;
        let parser = AAML::parse(content).expect("Error parsing config");

        assert_eq!(parser.find_obj("key1").unwrap().as_str(), "value1");
        assert_eq!(parser.find_obj("key2").unwrap().as_str(), "value2");
        assert_eq!(parser.find_obj("key3").unwrap().as_str(), "value3");
    }

    #[test]
    fn test_nested_quotes_behavior() {
        let content = r#"key = "'inner quotes'""#;
        let parser = AAML::parse(content).expect("Error parsing config");
        assert_eq!(parser.find_obj("key").unwrap().as_str(), "'inner quotes'");
    }

    #[test]
    fn test_comments_basic() {
        let content = "key = value # Это комментарий";
        let parser = AAML::parse(content).expect("Should parse");
        assert_eq!(parser.find_obj("key").unwrap().as_str(), "value");
    }

    #[test]
    fn test_comments_inside_quotes() {
        let content = r#"key = "value # not a comment""#;
        let parser = AAML::parse(content).expect("Should parse");
        assert_eq!(parser.find_obj("key").unwrap().as_str(), "value # not a comment");
    }

    #[test]
    fn test_comments_mixed() {
        let content = r#"
            key1 = "val1" # comment 1
            key2 = 'val # 2' # comment 2
            # full line comment
            key3 = val3
        "#;
        let parser = AAML::parse(content).expect("Should parse");

        assert_eq!(parser.find_obj("key1").unwrap().as_str(), "val1");
        assert_eq!(parser.find_obj("key2").unwrap().as_str(), "val # 2");
        assert_eq!(parser.find_obj("key3").unwrap().as_str(), "val3");
    }

    #[test]
    fn test_merge_content() {
        let mut aaml = AAML::new();
        aaml.merge_content("a = 1").expect("Merge failed");
        aaml.merge_content("b = 2").expect("Merge failed");
        aaml.merge_content("a = 3").expect("Merge failed");

        assert_eq!(aaml.find_obj("a").unwrap().as_str(), "3");
        assert_eq!(aaml.find_obj("b").unwrap().as_str(), "2");
    }

    #[test]
    fn test_add_trait() {
        let aaml1 = AAML::parse("a = 1").unwrap();
        let aaml2 = AAML::parse("b = 2").unwrap();

        let res = aaml1 + aaml2;
        assert_eq!(res.find_obj("a").unwrap().as_str(), "1");
        assert_eq!(res.find_obj("b").unwrap().as_str(), "2");
    }

    #[test]
    fn test_add_assign_trait() {
        let mut aaml1 = AAML::parse("a = 1").unwrap();
        let aaml2 = AAML::parse("a = 2\nb = 3").unwrap();

        aaml1 += aaml2;
        assert_eq!(aaml1.find_obj("a").unwrap().as_str(), "2");
        assert_eq!(aaml1.find_obj("b").unwrap().as_str(), "3");
    }

    #[test]
    fn test_reverse_lookup() {
        let content = "username = admin";
        let parser = AAML::parse(content).expect("Should parse");

        assert_eq!(parser.find_obj("username").unwrap().as_str(), "admin");
        assert_eq!(parser.find_obj("admin").unwrap().as_str(), "username");
    }

    #[test]
    fn test_empty_lines_and_whitespaces() {
        let content = "
            key1   =    val1
            key2=val2
        ";
        let parser = AAML::parse(content).expect("Should handle whitespace");
        assert_eq!(parser.find_obj("key1").unwrap().as_str(), "val1");
        assert_eq!(parser.find_obj("key2").unwrap().as_str(), "val2");
    }

    #[test]
    fn test_missing_equals_error_details() {
        let content = "key_without_value";
        let err = AAML::parse(content).unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Missing assignment operator"));
        assert!(err_msg.contains("key_without_value"));
    }

    #[test]
    fn test_deep_find_circular() {
        let content = "a=b\nb=c\nc=a";
        let parser = AAML::parse(content).expect("Parsed");
        let res = parser.find_deep("a");
        assert!(res.is_some());
        assert_eq!(res.unwrap().as_str(), "c");
    }
}