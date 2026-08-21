#[cfg(test)]
mod tests {
    use crate::aaml::AAML;
    use crate::builder::AAMBuilder;
    use std::fs;

    // ─────────────────────────────────────────────────────────────
    //  @derive tests
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn test_derive_inherits_keys() {
        let base_file = "test_derive_base_inherit.aam";
        let mut base = AAMBuilder::new();
        base.add_line("base_key", "base_val");
        base.add_line("shared_key", "from_base");
        base.to_file(base_file).unwrap();

        let content = format!(
            "@derive {base_file}\nchild_key = child_val\n"
        );
        let parser = AAML::parse(&content);
        let _ = fs::remove_file(base_file);
        let parser = parser.expect("Should parse @derive");

        assert_eq!(parser.find_obj("base_key").unwrap().as_str(), "base_val");
        assert_eq!(parser.find_obj("child_key").unwrap().as_str(), "child_val");
    }

    #[test]
    fn test_derive_does_not_overwrite_child_keys() {
        let base_file = "test_derive_no_overwrite.aam";
        let mut base = AAMBuilder::new();
        base.add_line("shared_key", "from_base");
        base.to_file(base_file).unwrap();

        let content = format!(
            "shared_key = from_child\n@derive {base_file}\n"
        );
        let parser = AAML::parse(&content);
        let _ = fs::remove_file(base_file);
        let parser = parser.expect("Should parse @derive");

        // Child key must NOT be overwritten by base
        assert_eq!(parser.find_obj("shared_key").unwrap().as_str(), "from_child");
    }

    #[test]
    fn test_derive_with_quoted_path() {
        let base_file = "test_derive_quoted.aam";
        let mut base = AAMBuilder::new();
        base.add_line("q_key", "q_val");
        base.to_file(base_file).unwrap();

        let content = format!(r#"@derive "{base_file}""#);
        let parser = AAML::parse(&content);
        let _ = fs::remove_file(base_file);
        let parser = parser.expect("Should parse @derive with quoted path");

        assert_eq!(parser.find_obj("q_key").unwrap().as_str(), "q_val");
    }

    #[test]
    fn test_derive_inherits_schemas() {
        let base_file = "test_derive_schemas.aam";
        let mut base = AAMBuilder::new();
        base.add_raw("@schema Point { x: f64, y: f64 }");
        base.add_line("origin", "0.0, 0.0");
        base.to_file(base_file).unwrap();

        let content = format!("@derive {base_file}\n");
        let parser = AAML::parse(&content);
        let _ = fs::remove_file(base_file);
        let parser = parser.expect("Should parse @derive with schemas");

        // schema "Point" must be inherited
        assert!(parser.get_schema("Point").is_some());
        assert_eq!(parser.find_obj("origin").unwrap().as_str(), "0.0, 0.0");
    }

    #[test]
    fn test_derive_schema_not_overwritten_by_base() {
        let base_file = "test_derive_schema_nowipe.aam";
        let mut base = AAMBuilder::new();
        base.add_raw("@schema Config { timeout: i32 }");
        base.to_file(base_file).unwrap();

        // Child defines its own schema with same name but different fields
        let content = format!("@schema Config {{ timeout: f64, retries: i32 }}\n@derive {base_file}\n");
        let parser = AAML::parse(&content);
        let _ = fs::remove_file(base_file);
        let parser = parser.expect("Should parse @derive with conflicting schemas");

        let schema = parser.get_schema("Config").expect("Schema must exist");
        // Child's schema must win — has "retries" field
        assert!(schema.fields.contains_key("retries"));
        assert_eq!(schema.fields.get("timeout").unwrap(), "f64");
    }

    #[test]
    fn test_derive_missing_file_error() {
        let content = "@derive non_existent_file_xyz.aam\n";
        let result = AAML::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_empty_path_error() {
        let content = "@derive \n";
        let result = AAML::parse(content);
        assert!(result.is_err());
    }

    // ─────────────────────────────────────────────────────────────
    //  @schema tests
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn test_schema_registers_fields() {
        let content = "@schema Player { name: string, score: i32, health: f64 }";
        let parser = AAML::parse(content).expect("Should parse schema");

        let schema = parser.get_schema("Player").expect("Schema 'Player' must exist");
        assert_eq!(schema.fields.get("name").unwrap(), "string");
        assert_eq!(schema.fields.get("score").unwrap(), "i32");
        assert_eq!(schema.fields.get("health").unwrap(), "f64");
    }

    #[test]
    fn test_schema_empty_body() {
        let content = "@schema Empty {  }";
        let parser = AAML::parse(content).expect("Should parse empty schema");
        let schema = parser.get_schema("Empty").expect("Schema 'Empty' must exist");
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn test_schema_missing_brace_error() {
        let result = AAML::parse("@schema Bad name: string }");
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_missing_closing_brace_error() {
        let result = AAML::parse("@schema Bad { name: string ");
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_empty_name_error() {
        let result = AAML::parse("@schema { name: string }");
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_bad_field_no_colon() {
        let result = AAML::parse("@schema Bad { name_string }");
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_multiple() {
        let content = r#"
            @schema Vec2 { x: f64, y: f64 }
            @schema Vec3 { x: f64, y: f64, z: f64 }
        "#;
        let parser = AAML::parse(content).expect("Should parse multiple schemas");
        assert!(parser.get_schema("Vec2").is_some());
        assert!(parser.get_schema("Vec3").is_some());

        let v3 = parser.get_schema("Vec3").unwrap();
        assert!(v3.fields.contains_key("z"));
    }

    // ─────────────────────────────────────────────────────────────
    //  @type tests
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn test_type_registers_primitive_alias() {
        let content = "@type age = i32";
        let parser = AAML::parse(content).expect("Should parse @type");
        assert!(parser.get_type("age").is_some());
    }

    #[test]
    fn test_type_validates_valid_value() {
        let content = "@type count = i32";
        let parser = AAML::parse(content).expect("Should parse @type");
        assert!(parser.validate_value("count", "42").is_ok());
    }

    #[test]
    fn test_type_validates_invalid_value() {
        let content = "@type count = i32";
        let parser = AAML::parse(content).expect("Should parse @type");
        let result = parser.validate_value("count", "not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_builtin_math() {
        let content = "@type position = math::vector3";
        let parser = AAML::parse(content).expect("Should parse @type with builtin");
        assert!(parser.validate_value("position", "1.0, 2.0, 3.0").is_ok());
        assert!(parser.validate_value("position", "1.0, 2.0").is_err());
    }

    #[test]
    fn test_type_builtin_physics() {
        let content = "@type mass = physics::kilogram";
        let parser = AAML::parse(content).expect("Should parse @type physics");
        assert!(parser.validate_value("mass", "75.5").is_ok());
        assert!(parser.validate_value("mass", "bad").is_err());
    }

    #[test]
    fn test_type_builtin_time() {
        let content = "@type created_at = time::datetime";
        let parser = AAML::parse(content).expect("Should parse @type time");
        assert!(parser.validate_value("created_at", "2024-01-15").is_ok());
        assert!(parser.validate_value("created_at", "bad-date").is_err());
    }

    #[test]
    fn test_type_missing_name_error() {
        let result = AAML::parse("@type = i32");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_missing_definition_error() {
        let result = AAML::parse("@type mytype = ");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_value_unknown_type() {
        let parser = AAML::parse("").unwrap();
        let result = parser.validate_value("unknown_type", "value");
        assert!(result.is_err());
    }
}

