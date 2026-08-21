#[cfg(test)]
mod tests {
    use aam_rs::aaml::AAML;
    use aam_rs::builder::AAMBuilder;
    use std::fs;


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

    #[test]
    fn test_import_functionality() {
        let sub_file = "test_sub_config.aam";
        let mut builder = AAMBuilder::new();
        builder.add_line("sub_key", "sub_value");
        builder.to_file(sub_file).unwrap();

        let content = format!("main_key = main_value\n@import {}", sub_file);

        let parser = AAML::parse(&content);

        let _ = fs::remove_file(sub_file);

        let parser = parser.expect("Should parse import");

        assert_eq!(parser.find_obj("main_key").unwrap().as_str(), "main_value");
        assert_eq!(parser.find_obj("sub_key").unwrap().as_str(), "sub_value");
    }

    #[test]
    fn test_recursive_import() {
        let file1 = "rec_import_1.aam";
        let file2 = "rec_import_2.aam";

        let mut b1 = AAMBuilder::new();
        b1.import(file2);
        b1.add_line("key1", "val1");
        b1.to_file(file1).unwrap();

        let mut b2 = AAMBuilder::new();
        b2.add_line("key2", "val2");
        b2.to_file(file2).unwrap();

        let parser = AAML::load(file1);

        let _ = fs::remove_file(file1);
        let _ = fs::remove_file(file2);

        let parser = parser.expect("Should load recursive imports");

        assert_eq!(parser.find_obj("key1").unwrap().as_str(), "val1");
        assert_eq!(parser.find_obj("key2").unwrap().as_str(), "val2");
    }

    #[test]
    fn test_import_with_quotes() {
        let sub_file = "quoted_import.aam";
        let mut b = AAMBuilder::new();
        b.add_line("q_key", "q_val");
        b.to_file(sub_file).unwrap();

        let content = format!(r#"@import "{}""#, sub_file);

        let parser = AAML::parse(&content);
        let _ = fs::remove_file(sub_file);

        let parser = parser.expect("Should parse quoted import path");
        assert_eq!(parser.find_obj("q_key").unwrap().as_str(), "q_val");
    }
}