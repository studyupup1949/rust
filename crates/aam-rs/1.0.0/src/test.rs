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
        let parser = AAML::parse(TEST_CONFIG).expect("Ошибка парсинга конфига");
        let res = parser.find_obj("a").expect("Должен найти 'a'");
        assert_eq!(res, "b");
    }

    #[test]
    fn test_not_found() {
        let parser = AAML::parse(TEST_CONFIG).expect("Ошибка парсинга конфига");
        assert!(parser.find_obj("unknown").is_none());
    }

    #[test]
    fn test_deref_behavior() {
        let parser = AAML::parse(TEST_CONFIG).expect("Ошибка парсинга конфига");
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
        let parser = AAML::parse(TEST_CONFIG).expect("Ошибка парсинга конфига");
        let res = parser.find_deep("c").expect("Должен найти 'c'");
        assert_eq!(res, "g");
    }

    #[test]
    fn test_find_deep_direct_loop() {
        let content = "key1=key1";
        let aaml = AAML::parse(content).expect("Ошибка парсинга");

        let result = aaml.find_deep("key1");
        println!("{:?}", result);
        assert_eq!(result.unwrap().as_str(), "key1");
    }

    #[test]
    fn test_find_deep_indirect_loop() {
        let content = "a=b\nb=a";
        let aaml = AAML::parse(content).expect("Ошибка парсинга");

        let result = aaml.find_deep("a");
        println!("{:?}", result);
        assert_eq!(result.unwrap().as_str(), "b");
    }

    #[test]
    fn test_find_deep_long_chain_with_loop() {
        let content = "start=mid\nmid=end\nend=mid";
        let aaml = AAML::parse(content).expect("Ошибка парсинга");

        let result = aaml.find_deep("start");
        assert!(result.is_some());
    }

    #[test]
    fn test_find_deep_no_loop() {
        let content = "a=b\nb=c\nc=final";
        let aaml = AAML::parse(content).expect("Ошибка парсинга");

        let result = aaml.find_deep("a");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_error_missing_equals() {
        let content = "invalid_line_without_equals";
        let res = AAML::parse(content);

        assert!(res.is_err());

        let err = res.unwrap_err();
        println!("{}", err);
    }

    #[test]
    fn test_parse_error_empty_key() {
        let content = "= value";
        let res = AAML::parse(content);
        assert!(res.is_err());
    }
}