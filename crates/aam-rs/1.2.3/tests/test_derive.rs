#[cfg(test)]
mod tests {
    use aam_rs::aaml::AAML;
    use aam_rs::builder::{AAMBuilder, SchemaField};
    use aam_rs::error::AamlError;
    use std::fs;
    use std::collections::HashMap;

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
        // Schema 'Point' declares x and y — both must be present so that the
        // completeness check inside @derive succeeds.
        base.schema("Point", [SchemaField::required("x", "f64"), SchemaField::required("y", "f64")]);
        base.add_line("x", "1.0");
        base.add_line("y", "2.0");
        base.add_line("origin", "0.0, 0.0");
        base.to_file(base_file).unwrap();

        let content = format!("@derive {base_file}\n");
        let parser = AAML::parse(&content);
        let _ = fs::remove_file(base_file);
        let parser = parser.expect("Should parse @derive with schemas");

        // schema "Point" must be inherited
        assert!(parser.get_schema("Point").is_some());
        assert_eq!(parser.find_obj("origin").unwrap().as_str(), "0.0, 0.0");
        assert_eq!(parser.find_obj("x").unwrap().as_str(), "1.0");
        assert_eq!(parser.find_obj("y").unwrap().as_str(), "2.0");
    }

    #[test]
    fn test_derive_schema_not_overwritten_by_base() {
        let base_file = "test_derive_schema_nowipe.aam";
        let mut base = AAMBuilder::new();
        // Base has Config with only 'timeout: i32'. It must supply that field.
        base.schema("Config", [SchemaField::required("timeout", "i32")]);
        base.add_line("timeout", "30");
        base.to_file(base_file).unwrap();

        // Child defines its own Config schema with MORE fields and must supply them all.
        // Child's schema wins on name conflict. After @derive both 'timeout' and 'retries'
        // must be present (child supplies both, base would only supply 'timeout').
        let content = format!(
            "@schema Config {{ timeout: f64, retries: i32 }}\ntimeout = 5.0\nretries = 3\n@derive {base_file}\n"
        );
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


    #[test]
    fn test_schema_field_valid_type() {
        let content = "@schema Config { retries: i32 }\nretries = 5\n";
        let result = AAML::parse(content);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.err());
        let parser = result.unwrap();
        assert_eq!(parser.find_obj("retries").unwrap().as_str(), "5");
    }

    #[test]
    fn test_schema_field_invalid_type() {
        let content = "@schema Config { retries: i32 }\nretries = hello\n";
        let result = AAML::parse(content);
        assert!(result.is_err(), "Expected Err for invalid i32");
        let err = result.unwrap_err();
        assert!(
            matches!(err, AamlError::SchemaValidationError { .. }),
            "Expected SchemaValidationError, got: {}", err
        );
    }

    #[test]
    fn test_schema_unknown_type_error() {
        let content = "@schema Config { speed: unicorn_type }\nspeed = 42\n";
        let result = AAML::parse(content);
        assert!(result.is_err(), "Expected Err for unknown type");
        let err = result.unwrap_err();
        assert!(
            matches!(err, AamlError::SchemaValidationError { .. }),
            "Expected SchemaValidationError, got: {}", err
        );
    }

    #[test]
    fn test_schema_string_type_always_valid() {
        let content = "@schema Config { name: string }\nname = hello world 123!@#\n";
        let result = AAML::parse(content);
        assert!(result.is_ok(), "Expected Ok for string type, got: {:?}", result.err());
    }

    #[test]
    fn test_schema_f64_valid() {
        let content = "@schema Config { ratio: f64 }\nratio = 3.14\n";
        assert!(AAML::parse(content).is_ok());
    }

    #[test]
    fn test_schema_f64_invalid() {
        let content = "@schema Config { ratio: f64 }\nratio = not_a_float\n";
        let result = AAML::parse(content);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AamlError::SchemaValidationError { .. }));
    }

    #[test]
    fn test_schema_bool_valid() {
        let content = "@schema Config { enabled: bool }\nenabled = true\n";
        assert!(AAML::parse(content).is_ok());
    }

    #[test]
    fn test_schema_bool_invalid() {
        let content = "@schema Config { enabled: bool }\nenabled = yes\n";
        let result = AAML::parse(content);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AamlError::SchemaValidationError { .. }));
    }

    #[test]
    fn test_schema_custom_type_alias_valid() {
        let content = "@type age = i32\n@schema Person { age: age }\nage = 25\n";
        assert!(AAML::parse(content).is_ok());
    }

    #[test]
    fn test_schema_custom_type_alias_invalid() {
        let content = "@type age = i32\n@schema Person { age: age }\nage = twenty-five\n";
        let result = AAML::parse(content);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AamlError::SchemaValidationError { .. }));
    }
    
    #[test]
    fn test_apply_schema_all_valid() {
        let content = "@schema Player { name: string, score: i32, health: f64 }";
        let parser = AAML::parse(content).unwrap();

        let mut data = HashMap::new();
        data.insert("name".to_string(), "Alice".to_string());
        data.insert("score".to_string(), "100".to_string());
        data.insert("health".to_string(), "98.5".to_string());

        assert!(parser.apply_schema("Player", &data).is_ok());
    }

    #[test]
    fn test_apply_schema_missing_field() {
        let content = "@schema Player { name: string, score: i32 }";
        let parser = AAML::parse(content).unwrap();

        let mut data = HashMap::new();
        data.insert("name".to_string(), "Alice".to_string());

        let result = parser.apply_schema("Player", &data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AamlError::SchemaValidationError { .. }));
    }

    #[test]
    fn test_apply_schema_wrong_value() {
        let content = "@schema Player { score: i32 }";
        let parser = AAML::parse(content).unwrap();

        let mut data = HashMap::new();
        data.insert("score".to_string(), "not_a_number".to_string());

        let result = parser.apply_schema("Player", &data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AamlError::SchemaValidationError { .. }));
    }

    #[test]
    fn test_apply_schema_not_found() {
        let parser = AAML::parse("").unwrap();
        let result = parser.apply_schema("NonExistent", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_validation_via_derive() {
        let base_file = "test_derive_schema_validation.aam";
        let mut base = AAMBuilder::new();
        base.schema("Config", [SchemaField::required("timeout", "i32")]);
        base.to_file(base_file).unwrap();

        let content = format!("@derive {base_file}\ntimeout = not_a_number\n");
        let result = AAML::parse(&content);
        let _ = fs::remove_file(base_file);

        assert!(result.is_err(), "Expected Err when derived schema type is violated");
    }

    #[test]
    fn test_derive_two_schema_selectors() {
        let base_file = "test_derive_two_selectors.aam";
        let mut base = AAMBuilder::new();
        base.schema("SchemaA", [SchemaField::required("a_val", "i32")]);
        base.schema("SchemaB", [SchemaField::required("b_val", "string")]);
        base.schema("SchemaC", [SchemaField::required("c_val", "f64")]);
        base.add_line("a_val", "1");
        base.add_line("b_val", "hello");
        base.to_file(base_file).unwrap();

        let content = format!(
            "@derive {base_file}::SchemaA::SchemaB\na_val = 42\nb_val = world\n"
        );
        let parser = AAML::parse(&content);
        let _ = fs::remove_file(base_file);
        let parser = parser.expect("@derive with two selectors must succeed");

        assert!(parser.get_schema("SchemaA").is_some(), "SchemaA must be imported");
        assert!(parser.get_schema("SchemaB").is_some(), "SchemaB must be imported");
        assert!(parser.get_schema("SchemaC").is_none(), "SchemaC must NOT be imported");
    }

    #[test]
    fn test_derive_selector_child_values_win() {
        let base_file = "test_derive_selector_child_wins.aam";
        let mut base = AAMBuilder::new();
        base.schema("Config", [SchemaField::required("port", "i32"), SchemaField::required("host", "string")]);
        base.add_line("port", "8080");
        base.add_line("host", "base-host");
        base.to_file(base_file).unwrap();

        let content = format!(
            "port = 9090\nhost = child-host\n@derive {base_file}::Config\n"
        );
        let parser = AAML::parse(&content);
        let _ = fs::remove_file(base_file);
        let parser = parser.expect("derive with selector must succeed");

        assert_eq!(parser.find_obj("port").unwrap().as_str(), "9090", "child port wins");
        assert_eq!(parser.find_obj("host").unwrap().as_str(), "child-host", "child host wins");
    }

    #[test]
    fn test_derive_optional_field_absent_is_ok() {
        let base_file = "test_derive_optional_absent.aam";
        let mut base = AAMBuilder::new();
        base.schema("Config", [SchemaField::required("timeout", "i32"), SchemaField::optional("retries", "i32")]);
        base.add_line("timeout", "30");
        base.to_file(base_file).unwrap();

        let content = format!("@derive {base_file}\ntimeout = 60\n");
        let result = AAML::parse(&content);
        let _ = fs::remove_file(base_file);

        assert!(result.is_ok(), "optional field absent must not be an error: {:?}", result.err());
        let parser = result.unwrap();
        let schema = parser.get_schema("Config").unwrap();
        assert!(schema.is_optional("retries"), "retries must be optional");
        assert!(parser.find_obj("retries").is_none(), "retries must not be set");
    }

    #[test]
    fn test_derive_optional_field_present_validated() {
        let base_file = "test_derive_optional_valid.aam";
        let mut base = AAMBuilder::new();
        base.schema("Config", [SchemaField::required("timeout", "i32"), SchemaField::optional("retries", "i32")]);
        base.add_line("timeout", "30");
        base.to_file(base_file).unwrap();

        let content_ok = format!("@derive {base_file}\ntimeout = 10\nretries = 5\n");
        let result_ok = AAML::parse(&content_ok);

        let content_bad = format!("@derive {base_file}\ntimeout = 10\nretries = not_int\n");
        let result_bad = AAML::parse(&content_bad);

        let _ = fs::remove_file(base_file);

        assert!(result_ok.is_ok(), "valid optional field must pass: {:?}", result_ok.err());
        assert!(result_bad.is_err(), "invalid optional field must fail");
        assert!(matches!(
            result_bad.unwrap_err(),
            AamlError::SchemaValidationError { .. }
        ));
    }

    #[test]
    fn test_derive_nonexistent_selector_error() {
        let base_file = "test_derive_nonexist_selector.aam";
        let mut base = AAMBuilder::new();
        base.schema("RealSchema", [SchemaField::required("x", "i32")]);
        base.add_line("x", "1");
        base.to_file(base_file).unwrap();

        let content = format!("@derive {base_file}::NonExistent\n");
        let result = AAML::parse(&content);
        let _ = fs::remove_file(base_file);

        assert!(result.is_err(), "must fail for non-existent schema selector");
        assert!(matches!(result.unwrap_err(), AamlError::DirectiveError(..)));
    }

    #[test]
    fn test_schema_nested_in_schema() {
        let content = r#"
            @schema Point  { x: f64, y: f64 }
            @schema Circle { center: Point, radius: f64 }
        "#;
        let cfg = AAML::parse(content).unwrap();

        let mut ok: HashMap<String, String> = HashMap::new();
        ok.insert("center".into(), "{ x = 1.0, y = 2.5 }".into());
        ok.insert("radius".into(), "5.0".into());
        assert!(cfg.apply_schema("Circle", &ok).is_ok(), "valid Circle must pass");

        let mut bad_center: HashMap<String, String> = HashMap::new();
        bad_center.insert("center".into(), "{ x = not_a_float, y = 2.5 }".into());
        bad_center.insert("radius".into(), "5.0".into());
        let err = cfg.apply_schema("Circle", &bad_center).unwrap_err();
        assert!(matches!(err, AamlError::SchemaValidationError { .. }));
    }

    #[test]
    fn test_schema_nested_optional_field_absent() {
        let content = r#"
            @schema Item   { item_name: string, item_weight: f64, item_rare*: bool }
            @schema Weapon { base: Item, damage: i32 }
        "#;
        let cfg = AAML::parse(content).unwrap();

        let mut data: HashMap<String, String> = HashMap::new();
        // item_rare* не указан — это допустимо
        data.insert("base".into(), "{ item_name = Sword, item_weight = 2.5 }".into());
        data.insert("damage".into(), "50".into());

        assert!(cfg.apply_schema("Weapon", &data).is_ok());
    }

    #[test]
    fn test_list_of_strings_valid() {
        let content = "@schema Tags { name: string, items: list<string> }";
        let cfg = AAML::parse(content).unwrap();

        let mut data: HashMap<String, String> = HashMap::new();
        data.insert("name".into(), "my-tags".into());
        data.insert("items".into(), "[foo, bar, baz]".into());

        assert!(cfg.apply_schema("Tags", &data).is_ok());
    }

    #[test]
    fn test_list_of_i32_invalid_element() {
        let content = "@schema Scores { title: string, values: list<i32> }";
        let cfg = AAML::parse(content).unwrap();

        let mut data: HashMap<String, String> = HashMap::new();
        data.insert("title".into(), "test".into());
        data.insert("values".into(), "[1, 2, not_int, 4]".into());

        let err = cfg.apply_schema("Scores", &data).unwrap_err();
        assert!(matches!(err, AamlError::SchemaValidationError { .. }));
    }

    #[test]
    fn test_list_of_nested_schema_valid() {
        let content = r#"
            @schema Point  { x: f64, y: f64 }
            @schema Polyline { label: string, points: list<Point> }
        "#;
        let cfg = AAML::parse(content).unwrap();

        let mut data: HashMap<String, String> = HashMap::new();
        data.insert("label".into(), "shape".into());
        data.insert(
            "points".into(),
            "[{ x = 0.0, y = 0.0 }, { x = 1.0, y = 1.0 }, { x = 2.0, y = 0.0 }]".into(),
        );

        assert!(cfg.apply_schema("Polyline", &data).is_ok());
    }

    #[test]
    fn test_list_of_nested_schema_invalid_element() {
        let content = r#"
            @schema Point  { x: f64, y: f64 }
            @schema Polyline { label: string, points: list<Point> }
        "#;
        let cfg = AAML::parse(content).unwrap();

        let mut data: HashMap<String, String> = HashMap::new();
        data.insert("label".into(), "bad".into());
        data.insert(
            "points".into(),
            "[{ x = 0.0, y = 0.0 }, { x = oops, y = 1.0 }]".into(),
        );

        let err = cfg.apply_schema("Polyline", &data).unwrap_err();
        assert!(matches!(err, AamlError::SchemaValidationError { .. }));
    }

    #[test]
    fn test_derive_with_quoted_selector() {
        let base_file = "test_derive_quoted_selector.aam";
        let mut base = AAMBuilder::new();
        base.schema("Quoted", [SchemaField::required("val", "string")]);
        base.add_line("val", "hello");
        base.to_file(base_file).unwrap();

        let content = format!("@derive \"{base_file}\"::Quoted\nval = world\n");
        let result = AAML::parse(&content);
        let _ = fs::remove_file(base_file);

        let parser = result.expect("quoted path + selector must work");
        assert!(parser.get_schema("Quoted").is_some());
        assert_eq!(parser.find_obj("val").unwrap().as_str(), "world");
    }

    #[test]
    fn test_schema_all_optional_fields() {
        let content = "@schema Meta { author*: string, version*: string, tags*: list<string> }";
        let cfg = AAML::parse(content).expect("All-optional schema must parse");
        let schema = cfg.get_schema("Meta").unwrap();
        assert!(schema.is_optional("author"));
        assert!(schema.is_optional("version"));
        assert!(schema.is_optional("tags"));

        let result = cfg.apply_schema("Meta", &HashMap::new());
        assert!(result.is_ok(), "empty data for all-optional schema must be ok");
    }

    #[test]
    fn test_derive_missing_required_field_in_child_schema() {
        let base_file = "test_derive_missing_required.aam";
        let mut base = AAMBuilder::new();
        base.add_line("some_key", "some_val");
        base.to_file(base_file).unwrap();

        let content = format!(
            "@schema Required {{ must_exist: string }}\n@derive {base_file}\n"
        );
        let result = AAML::parse(&content);
        let _ = fs::remove_file(base_file);

        assert!(result.is_err(), "missing required field must cause error");
        assert!(matches!(result.unwrap_err(), AamlError::SchemaValidationError { .. }));
    }
}
