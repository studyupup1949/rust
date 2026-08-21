#[cfg(test)]
mod tests {
    use aam_rs::aaml::parsing::{is_inline_object, parse_inline_object};
    use aam_rs::aaml::parsing::strip_comment;

    #[test]
    fn color_not_stripped() {
        assert_eq!(strip_comment("tint = #ff6600"), "tint = #ff6600");
        assert_eq!(strip_comment("tint=#ff6600"), "tint=#ff6600");
    }

    #[test]
    fn comment_after_space_stripped() {
        assert_eq!(strip_comment("key = value # comment").trim(), "key = value");
    }

    #[test]
    fn quoted_hash_preserved() {
        assert_eq!(strip_comment(r#"key = "val # not comment""#), r#"key = "val # not comment""#);
    }

    #[test]
    fn inline_object_parsed() {
        let result = parse_inline_object("{ x = 1.0, y = 2.0 }").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|(k, v)| k == "x" && v == "1.0"));
        assert!(result.iter().any(|(k, v)| k == "y" && v == "2.0"));
    }

    #[test]
    fn inline_object_colon_separator() {
        let result = parse_inline_object("{ name: Alice, score: 100 }").unwrap();
        assert!(result.iter().any(|(k, v)| k == "name" && v == "Alice"));
    }

    #[test]
    fn inline_object_quoted_values() {
        let result = parse_inline_object(r#"{ name = "Alice Doe", active = true }"#).unwrap();
        assert!(result.iter().any(|(k, v)| k == "name" && v == "Alice Doe"));
    }

    #[test]
    fn is_inline_object_detection() {
        assert!(is_inline_object("{ x = 1 }"));
        assert!(!is_inline_object("[1, 2, 3]"));
        assert!(!is_inline_object("hello"));
    }

    #[test]
    fn inline_object_nested_preserves_commas() {
        let result = parse_inline_object("{ base = { x = 1, y = 2 }, z = 3 }").unwrap();
        assert_eq!(result.len(), 2);
        let base = result.iter().find(|(k, _)| k == "base").unwrap();
        assert_eq!(base.1, "{ x = 1, y = 2 }");
        let z = result.iter().find(|(k, _)| k == "z").unwrap();
        assert_eq!(z.1, "3");
    }

    #[test]
    fn inline_object_nested_list_value() {
        let result = parse_inline_object("{ tags = [a, b, c], name = test }").unwrap();
        assert_eq!(result.len(), 2);
        let tags = result.iter().find(|(k, _)| k == "tags").unwrap();
        assert_eq!(tags.1, "[a, b, c]");
    }
}

