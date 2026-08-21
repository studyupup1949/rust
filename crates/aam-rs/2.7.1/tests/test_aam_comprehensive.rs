#[cfg(test)]
mod tests {
    use aam_rs::aam::AAM;

    #[test]
    fn parse_basic_pairs_and_keys() {
        let doc = AAM::parse("a = 1\nb = 2\nc = 3").expect("parse should succeed");
        assert_eq!(doc.get("a"), Some("1"));
        assert_eq!(doc.get("b"), Some("2"));
        assert_eq!(doc.get("c"), Some("3"));
        assert_eq!(doc.keys().len(), 3);
    }

    #[test]
    fn parse_supports_comments_and_colors() {
        let content = "tint = #ff6600\nname = test # comment";
        let doc = AAM::parse(content).expect("parse should succeed");
        assert_eq!(doc.get("tint"), Some("#ff6600"));
        assert_eq!(doc.get("name"), Some("test"));
    }

    #[test]
    fn parse_schema_and_type_metadata() {
        let content = "@schema Server { host: string, port: i32 }\n@type port_alias = i32\nhost = localhost\nport = 8080";
        let doc = AAM::parse(content).expect("parse should succeed");

        assert!(doc.get_schema("Server").is_some());
        assert!(doc.get_type("port_alias").is_some());
    }

    #[test]
    fn parse_reports_errors_for_invalid_assignment() {
        let err = AAM::parse("invalid_line_without_equals")
            .expect_err("invalid assignment should fail")
            .into_iter()
            .next()
            .expect("error list should not be empty");
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }
}
