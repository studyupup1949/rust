#[cfg(test)]
mod tests {
    use aam_rs::aam::AAM;
    use aam_rs::found_value::FoundValue;

    const TEST_CONFIG: &str = "
        a = b
        c = d
        e = f
        d = g
    ";

    #[test]
    fn test_simple_find() {
        let parser = AAM::parse(TEST_CONFIG).expect("Error parsing config");
        let res = parser.get("a").expect("Should find 'a'");
        assert_eq!(res, "b");
    }

    #[test]
    fn test_not_found() {
        let parser = AAM::parse(TEST_CONFIG).expect("Error parsing config");
        assert!(parser.get("unknown").is_none());
    }

    #[test]
    fn test_deref_behavior() {
        let parser = AAM::parse(TEST_CONFIG).expect("Error parsing config");
        let res = parser.get("a").unwrap();

        assert_eq!(res.len(), 1);
        assert!(res.starts_with('b'));
    }

    #[test]
    fn test_display_trait() {
        let res = FoundValue::new("hello");
        let formatted = format!("{}", res);
        assert_eq!(formatted, "hello");
    }

    #[test]
    fn test_find_by_key_or_value() {
        let parser = AAM::parse(TEST_CONFIG).expect("Error parsing config");
        let by_key = parser.find("a");
        assert_eq!(by_key, vec![("a", "b")]);

        let by_value = parser.find("g");
        assert!(by_value.iter().any(|(k, _)| *k == "d"));
    }

    #[test]
    fn test_reverse_lookup() {
        let content = "username = admin\nrole = admin";
        let parser = AAM::parse(content).expect("Error parsing config");
        let keys = parser.reverse_search("admin");
        assert!(keys.contains(&"username"));
        assert!(keys.contains(&"role"));
    }

    #[test]
    fn test_deep_search() {
        let content = "service.host = localhost\nservice.port = 8080\nuser = root";
        let parser = AAM::parse(content).expect("Error parsing config");
        let result = parser.deep_search("service");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_find_by_predicate() {
        let parser = AAM::parse(TEST_CONFIG).expect("Error parsing config");
        let result = parser.find_by(|k, _| k.starts_with('l'));
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_error_missing_equals() {
        let content = "invalid_line_without_equals";
        let res = AAM::parse(content);
        assert!(res.is_err());
        let err = res.unwrap_err().into_iter().next().unwrap();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Parse error") || err_msg.contains("assignment"));
    }

    #[test]
    fn test_parse_error_empty_key() {
        let content = "= value";
        let res = AAM::parse(content);
        assert!(res.is_err());
    }

    #[test]
    fn test_quotes_stripping() {
        let content = r#"
            key1 = "value1"
            key2 = 'value2'
            key3 = value3
            key4 = value4
        "#;
        let parser = AAM::parse(content).expect("Error parsing config");

        assert_eq!(parser.get("key1").unwrap(), "\"value1\"");
        assert_eq!(parser.get("key2").unwrap(), "'value2'");
        assert_eq!(parser.get("key3").unwrap(), "value3");
    }

    #[test]
    fn test_nested_quotes_behavior() {
        let content = r#"key = "'inner quotes'""#;
        let parser = AAM::parse(content).expect("Error parsing config");
        assert_eq!(parser.get("key").unwrap(), "\"'inner quotes'\"");
    }

    #[test]
    fn test_comments_basic() {
        let content = "key = value # comment";
        let parser = AAM::parse(content).expect("Should parse");
        assert_eq!(parser.get("key").unwrap(), "value");
    }

    #[test]
    fn test_comments_inside_quotes() {
        let content = r#"key = "value # not a comment""#;
        let parser = AAM::parse(content).expect("Should parse");
        assert_eq!(parser.get("key").unwrap(), "\"value # not a comment\"");
    }

    #[test]
    fn test_comments_mixed() {
        let content = r#"
            key1 = "val1" # comment 1
            key2 = 'val # 2' # comment 2
            # full line comment
            key3 = val3
        "#;
        let parser = AAM::parse(content).expect("Should parse");

        assert_eq!(parser.get("key1").unwrap(), "\"val1\"");
        assert_eq!(parser.get("key2").unwrap(), "'val # 2'");
        assert_eq!(parser.get("key3").unwrap(), "val3");
    }

    #[test]
    fn test_to_map_and_keys() {
        let doc = AAM::parse("a = 1\nb = 2").unwrap();
        let map = doc.to_map();
        let keys = doc.keys();
        assert_eq!(map.get("a"), Some(&"1".to_string()));
        assert!(keys.contains(&"a"));
        assert!(keys.contains(&"b"));
    }

    #[test]
    fn test_empty_lines_and_whitespaces() {
        let content = "
        key1=val1
        key2=val2
        ";
        let parser = AAM::parse(content).expect("Should handle whitespace");
        assert_eq!(parser.get("key1").unwrap(), "val1");
        assert_eq!(parser.get("key2").unwrap(), "val2");
        assert_eq!(parser.get("key2").unwrap(), "val2");
    }

    #[test]
    fn test_missing_equals_error_details() {
        let content = "key_without_value";
        let err = AAM::parse(content).unwrap_err().into_iter().next().unwrap();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Parse error") || err_msg.contains("assignment"));
        assert!(!err_msg.is_empty());
    }

    #[test]
    fn test_hex_color_value() {
        let doc = AAM::parse("tint = #ff6600").expect("Parsed");
        assert_eq!(doc.get("tint"), Some("#ff6600"));
    }
}
