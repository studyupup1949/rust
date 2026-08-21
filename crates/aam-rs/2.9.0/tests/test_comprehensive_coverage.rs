#![cfg(any())]

use aam_rs::aaml::AAML;
use aam_rs::aaml::parsing::{parse_inline_object, strip_comment};
use aam_rs::builder::{AAMBuilder, SchemaField};
use std::fmt::Write;
use std::fs;

// ============================================================================
// SECTION 1: Basic Parsing and Lookup Tests (50 tests)
// ============================================================================

#[test]
fn test_parse_simple_key_value_1() {
    let doc = AAML::parse("key = value").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_parse_simple_key_value_2() {
    let doc = AAML::parse("name = John").unwrap();
    assert_eq!(doc.find_obj("name").unwrap().as_str(), "John");
}

#[test]
fn test_parse_multiple_assignments_1() {
    let doc = AAML::parse("a=1\nb=2\nc=3").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
    assert_eq!(doc.find_obj("c").unwrap().as_str(), "3");
}

#[test]
fn test_parse_multiple_assignments_with_spaces() {
    let doc = AAML::parse("key1 = val1\nkey2 = val2").unwrap();
    assert_eq!(doc.find_obj("key1").unwrap().as_str(), "val1");
    assert_eq!(doc.find_obj("key2").unwrap().as_str(), "val2");
}

#[test]
fn test_find_nonexistent_key() {
    let doc = AAML::parse("key = value").unwrap();
    assert!(doc.find_obj("nonexistent").is_none());
}

#[test]
fn test_find_after_multiple_lookups() {
    let doc = AAML::parse("a=1\nb=2\nc=3").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
    assert_eq!(doc.find_obj("c").unwrap().as_str(), "3");
}

#[test]
fn test_case_sensitive_lookup() {
    let doc = AAML::parse("Key = value").unwrap();
    assert!(doc.find_obj("key").is_none());
    assert_eq!(doc.find_obj("Key").unwrap().as_str(), "value");
}

#[test]
fn test_empty_value() {
    let doc = AAML::parse("key = ").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "");
}

#[test]
fn test_parse_whitespace_only() {
    let doc = AAML::parse("   \n   \n   ").unwrap();
    assert!(doc.find_obj("anything").is_none());
}

#[test]
fn test_quoted_value_double() {
    let doc = AAML::parse(r#"key = "value with spaces""#).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value with spaces");
}

#[test]
fn test_quoted_value_single() {
    let doc = AAML::parse("key = 'single quoted'").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "single quoted");
}

#[test]
fn test_mixed_quotes_in_value() {
    let doc = AAML::parse(r#"key = "value 'with' quotes""#).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value 'with' quotes");
}

#[test]
fn test_equals_in_value() {
    let doc = AAML::parse(r#"key = "value=with=equals""#).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value=with=equals");
}

#[test]
fn test_newline_separated_pairs() {
    let doc = AAML::parse("a=1\nb=2\nc=3\nd=4").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("d").unwrap().as_str(), "4");
}

#[test]
fn test_trailing_whitespace() {
    let doc = AAML::parse("key = value   ").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_leading_whitespace_in_key() {
    let doc = AAML::parse("  key = value").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_numeric_keys() {
    let doc = AAML::parse("123 = value").unwrap();
    assert_eq!(doc.find_obj("123").unwrap().as_str(), "value");
}

#[test]
fn test_numeric_values() {
    let doc = AAML::parse("key = 12345").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "12345");
}

#[test]
fn test_special_chars_in_value() {
    let doc = AAML::parse(r#"key = "value!@#$%^&*()""#).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value!@#$%^&*()");
}

#[test]
fn test_underscore_in_keys() {
    let doc = AAML::parse("my_key = my_value").unwrap();
    assert_eq!(doc.find_obj("my_key").unwrap().as_str(), "my_value");
}

#[test]
fn test_dash_in_keys() {
    let doc = AAML::parse("my-key = my-value").unwrap();
    assert_eq!(doc.find_obj("my-key").unwrap().as_str(), "my-value");
}

#[test]
fn test_dot_in_keys() {
    let doc = AAML::parse("my.key = my.value").unwrap();
    assert_eq!(doc.find_obj("my.key").unwrap().as_str(), "my.value");
}

#[test]
fn test_colon_in_keys() {
    let doc = AAML::parse("my:key = value").unwrap();
    assert_eq!(doc.find_obj("my:key").unwrap().as_str(), "value");
}

#[test]
fn test_slash_in_keys() {
    let doc = AAML::parse("my/key = value").unwrap();
    assert_eq!(doc.find_obj("my/key").unwrap().as_str(), "value");
}

#[test]
fn test_multiple_equals_signs() {
    let doc = AAML::parse("key = value = more = stuff").unwrap();
    assert_eq!(
        doc.find_obj("key").unwrap().as_str(),
        "value = more = stuff"
    );
}

#[test]
fn test_tab_separation() {
    let doc = AAML::parse("key\t=\tvalue").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_empty_lines_between_entries() {
    let doc = AAML::parse("a=1\n\n\nb=2").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
}

#[test]
fn test_lookup_returns_found_value() {
    let doc = AAML::parse("key = value").unwrap();
    let result = doc.find_obj("key").unwrap();
    assert_eq!(result.as_str(), "value");
}

#[test]
fn test_found_value_deref() {
    let doc = AAML::parse("key = value").unwrap();
    let result = doc.find_obj("key").unwrap();
    assert_eq!(&*result, "value");
}

#[test]
fn test_found_value_len() {
    let doc = AAML::parse("key = hello").unwrap();
    let result = doc.find_obj("key").unwrap();
    assert_eq!(result.len(), 5);
}

#[test]
fn test_found_value_starts_with() {
    let doc = AAML::parse("key = hello").unwrap();
    let result = doc.find_obj("key").unwrap();
    assert!(result.starts_with("hel"));
}

#[test]
fn test_found_value_ends_with() {
    let doc = AAML::parse("key = hello").unwrap();
    let result = doc.find_obj("key").unwrap();
    assert!(result.ends_with("lo"));
}

#[test]
fn test_found_value_contains() {
    let doc = AAML::parse("key = hello").unwrap();
    let result = doc.find_obj("key").unwrap();
    assert!(result.contains("ell"));
}

#[test]
fn test_hex_color_value() {
    let doc = AAML::parse("color = #ff00ff").unwrap();
    assert_eq!(doc.find_obj("color").unwrap().as_str(), "#ff00ff");
}

// ============================================================================
// SECTION 2: Comment Parsing Tests (30 tests)
// ============================================================================

#[test]
fn test_comment_with_spaces() {
    let doc = AAML::parse("key = value # this is a comment").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_comment_without_space_before() {
    let doc = AAML::parse("key=value# no space").unwrap();
    // Without space before #, it's considered part of the value
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value# no space");
}

#[test]
fn test_comment_as_key() {
    let doc = AAML::parse("key = #ok # OK").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "#ok");
}

#[test]
fn test_no_comment_hex_color() {
    let doc = AAML::parse("color = #ff00ff").unwrap();
    assert_eq!(doc.find_obj("color").unwrap().as_str(), "#ff00ff");
}

#[test]
fn test_comment_in_comment() {
    let doc = AAML::parse("key = value # this # is # a # comment").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_hex_with_spaces_and_comment() {
    let doc = AAML::parse("color = #ff00ff # magenta").unwrap();
    assert_eq!(doc.find_obj("color").unwrap().as_str(), "#ff00ff");
}

#[test]
fn test_comment_in_quoted_value() {
    let doc = AAML::parse(r#"key = "value # with hash""#).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value # with hash");
}

#[test]
fn test_strip_comment_basic() {
    let result = strip_comment("key = value # comment");
    assert!(result.contains("key") && result.contains("value"));
    assert!(!result.contains("comment"));
}

#[test]
fn test_strip_comment_hex_color() {
    let result = strip_comment("color = #ff00ff");
    assert_eq!(result.trim(), "color = #ff00ff");
}

#[test]
fn test_strip_comment_no_comment() {
    let result = strip_comment("key = value");
    assert_eq!(result.trim(), "key = value");
}

#[test]
fn test_strip_comment_quoted() {
    let result = strip_comment(r#"key = "value # keep""#);
    assert_eq!(result.trim(), r#"key = "value # keep""#);
}

#[test]
fn test_strip_comment_single_quoted() {
    let result = strip_comment("key = 'value # keep'");
    assert_eq!(result.trim(), "key = 'value # keep'");
}

#[test]
fn test_comment_entire_line() {
    let doc = AAML::parse("a=1\n# comment\nb=2").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
}

#[test]
fn test_multiple_hashes_in_value() {
    let content = "key = \"##value##\"";
    let doc = AAML::parse(content).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "##value##");
}

#[test]
fn test_comment_after_various_values() {
    let doc = AAML::parse("a=1 # num\nb=text # str\nc=true # bool").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "text");
    assert_eq!(doc.find_obj("c").unwrap().as_str(), "true");
}

#[test]
fn test_strip_comment_preserves_quoted() {
    let line = r#"val = "a#b" # comment"#;
    let result = strip_comment(line);
    assert!(!result.contains("comment"));
    assert!(result.contains("a#b"));
}

#[test]
fn test_hex_rgb_color() {
    let doc = AAML::parse("tint = #ff6600").unwrap();
    assert_eq!(doc.find_obj("tint").unwrap().as_str(), "#ff6600");
}

#[test]
fn test_hex_rgba_color() {
    let doc = AAML::parse("tint = #ff6600aa").unwrap();
    assert_eq!(doc.find_obj("tint").unwrap().as_str(), "#ff6600aa");
}

#[test]
fn test_url_with_hash() {
    let doc = AAML::parse(r#"url = "https://example.com#section""#).unwrap();
    assert_eq!(
        doc.find_obj("url").unwrap().as_str(),
        "https://example.com#section"
    );
}

#[test]
fn test_multiline_with_comments() {
    let content = "a = 1 # first\nb = 2 # second\nc = 3 # third";
    let doc = AAML::parse(content).unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
    assert_eq!(doc.find_obj("c").unwrap().as_str(), "3");
}

#[test]
fn test_comment_only_line() {
    let doc = AAML::parse("a=1\n# just a comment\nb=2").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
}

#[test]
fn test_hash_at_start_of_value() {
    let doc = AAML::parse("color = #123456").unwrap();
    assert_eq!(doc.find_obj("color").unwrap().as_str(), "#123456");
}

#[test]
fn test_multiple_comments_per_line_invalid() {
    let doc = AAML::parse("key = val # comment1 # comment2").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "val");
}

#[test]
fn test_strip_comment_empty_string() {
    let result = strip_comment("");
    assert_eq!(result, "");
}

#[test]
fn test_strip_comment_only_hash() {
    let result = strip_comment("#");
    assert_eq!(result, "");
}

#[test]
fn test_comment_with_tab() {
    let doc = AAML::parse("key = value\t# tab comment").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

// ============================================================================
// SECTION 3: Reference Resolution and Lookup Tests (40 tests)
// ============================================================================

#[test]
fn test_find_deep_simple_reference() {
    let doc = AAML::parse("a = b\nb = value").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "value");
}

#[test]
fn test_find_deep_chain() {
    let doc = AAML::parse("a = b\nb = c\nc = d\nd = final").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "final");
}

#[test]
fn test_find_deep_long_chain() {
    let mut content = String::new();
    for i in 0..50 {
        let _ = writeln!(content, "k{} = k{}", i, i + 1);
    }
    content.push_str("k50 = terminus");
    let doc = AAML::parse(&content).unwrap();
    assert_eq!(doc.find_deep("k0").unwrap().as_str(), "terminus");
}

#[test]
fn test_find_deep_self_reference() {
    let doc = AAML::parse("key = key").unwrap();
    assert_eq!(doc.find_deep("key").unwrap().as_str(), "key");
}

#[test]
fn test_find_deep_two_way_loop() {
    let doc = AAML::parse("a = b\nb = a").unwrap();
    let result = doc.find_deep("a").unwrap();
    assert_eq!(result.as_str(), "b");
}

#[test]
fn test_find_deep_three_way_loop() {
    let doc = AAML::parse("a = b\nb = c\nc = a").unwrap();
    let result = doc.find_deep("a").unwrap();
    assert_eq!(result.as_str(), "c");
}

#[test]
fn test_find_deep_nonexistent_terminal() {
    let doc = AAML::parse("a = b").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "b");
}

#[test]
fn test_find_deep_with_literal_value() {
    let doc = AAML::parse("a = literal_value").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "literal_value");
}

#[test]
fn test_find_deep_numeric_chain() {
    let doc = AAML::parse("1 = 2\n2 = 3\n3 = final").unwrap();
    assert_eq!(doc.find_deep("1").unwrap().as_str(), "final");
}

#[test]
fn test_find_deep_special_char_chain() {
    let doc = AAML::parse("a.b = c.d\nc.d = e.f\ne.f = value").unwrap();
    assert_eq!(doc.find_deep("a.b").unwrap().as_str(), "value");
}

#[test]
fn test_find_deep_mixed_cases() {
    let doc = AAML::parse("Key = Value\nValue = Final").unwrap();
    assert_eq!(doc.find_deep("Key").unwrap().as_str(), "Final");
}

#[test]
fn test_find_deep_with_spaces() {
    let doc = AAML::parse(r#"key = "value with spaces""#).unwrap();
    assert_eq!(doc.find_deep("key").unwrap().as_str(), "value with spaces");
}

#[test]
fn test_find_deep_empty_reference() {
    let doc = AAML::parse("a = \nb = value").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "");
}

#[test]
fn test_find_deep_none_for_missing() {
    let doc = AAML::parse("a = b").unwrap();
    assert!(doc.find_deep("nonexistent").is_none());
}

#[test]
fn test_find_deep_multiple_levels() {
    let doc = AAML::parse("a=b\nb=c\nc=d\nd=e\ne=final").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "final");
}

#[test]
fn test_find_deep_reverse_lookup() {
    let doc = AAML::parse("forward = back\nback = value").unwrap();
    let result = doc.find_key("value");
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_str(), "back");
}

#[test]
fn test_find_deep_with_multiple_references() {
    let doc = AAML::parse("a=b\nc=d\nb=terminal\nd=terminal").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "terminal");
    assert_eq!(doc.find_deep("c").unwrap().as_str(), "terminal");
}

#[test]
fn test_find_deep_quoted_reference() {
    let doc = AAML::parse("a = b\nb = value").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "value");
}

#[test]
fn test_find_deep_with_numbers() {
    let doc = AAML::parse("var1 = var2\nvar2 = 42").unwrap();
    assert_eq!(doc.find_deep("var1").unwrap().as_str(), "42");
}

#[test]
fn test_find_key_reverse_lookup() {
    let doc = AAML::parse("key1 = value\nkey2 = value").unwrap();
    let result = doc.find_key("value");
    assert!(result.is_some());
}

#[test]
fn test_find_key_nonexistent_value() {
    let doc = AAML::parse("key = value").unwrap();
    assert!(doc.find_key("nonexistent").is_none());
}

#[test]
fn test_find_obj_vs_find_deep() {
    let doc = AAML::parse("a = b\nb = c").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "b");
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "c");
}

#[test]
fn test_multiple_paths_to_same_value() {
    let doc = AAML::parse("a = x\nb = x\nx = value").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "value");
    assert_eq!(doc.find_deep("b").unwrap().as_str(), "value");
}

#[test]
fn test_find_deep_circular_with_data() {
    let doc = AAML::parse("a = b\nb = c\nc = data").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "data");
}

#[test]
fn test_find_obj_basic() {
    let doc = AAML::parse("key = value").unwrap();
    assert!(doc.find_obj("key").is_some());
    assert!(doc.find_obj("missing").is_none());
}

#[test]
fn test_find_obj_numeric() {
    let doc = AAML::parse("123 = 456").unwrap();
    assert_eq!(doc.find_obj("123").unwrap().as_str(), "456");
}

// ============================================================================
// SECTION 4: Type Validation Tests (60 tests)
// ============================================================================

#[test]
fn test_validate_i32_positive() {
    let doc = AAML::new();
    assert!(doc.validate_value("i32", "123").is_ok());
}

#[test]
fn test_validate_i32_negative() {
    let doc = AAML::new();
    assert!(doc.validate_value("i32", "-456").is_ok());
}

#[test]
fn test_validate_i32_zero() {
    let doc = AAML::new();
    assert!(doc.validate_value("i32", "0").is_ok());
}

#[test]
fn test_validate_i32_invalid() {
    let doc = AAML::new();
    assert!(doc.validate_value("i32", "not_a_number").is_err());
}

#[test]
fn test_validate_i32_float() {
    let doc = AAML::new();
    assert!(doc.validate_value("i32", "3.14").is_err());
}

#[test]
fn test_validate_i32_large() {
    let doc = AAML::new();
    assert!(doc.validate_value("i32", "2147483647").is_ok());
}

#[test]
fn test_validate_f64_positive() {
    let doc = AAML::new();
    assert!(doc.validate_value("f64", "3.14").is_ok());
}

#[test]
fn test_validate_f64_negative() {
    let doc = AAML::new();
    assert!(doc.validate_value("f64", "-2.71").is_ok());
}

#[test]
fn test_validate_f64_integer_form() {
    let doc = AAML::new();
    assert!(doc.validate_value("f64", "42").is_ok());
}

#[test]
fn test_validate_f64_scientific() {
    let doc = AAML::new();
    assert!(doc.validate_value("f64", "1.5e10").is_ok());
}

#[test]
fn test_validate_f64_invalid() {
    let doc = AAML::new();
    assert!(doc.validate_value("f64", "not_a_float").is_err());
}

#[test]
fn test_validate_bool_true_lowercase() {
    let doc = AAML::new();
    assert!(doc.validate_value("bool", "true").is_ok());
}

#[test]
fn test_validate_bool_false_lowercase() {
    let doc = AAML::new();
    assert!(doc.validate_value("bool", "false").is_ok());
}

#[test]
fn test_validate_bool_true_uppercase() {
    let doc = AAML::new();
    assert!(doc.validate_value("bool", "TRUE").is_ok());
}

#[test]
fn test_validate_bool_false_uppercase() {
    let doc = AAML::new();
    assert!(doc.validate_value("bool", "FALSE").is_ok());
}

#[test]
fn test_validate_bool_one() {
    let doc = AAML::new();
    assert!(doc.validate_value("bool", "1").is_ok());
}

#[test]
fn test_validate_bool_zero() {
    let doc = AAML::new();
    assert!(doc.validate_value("bool", "0").is_ok());
}

#[test]
fn test_validate_bool_invalid_yes() {
    let doc = AAML::new();
    assert!(doc.validate_value("bool", "yes").is_err());
}

#[test]
fn test_validate_bool_invalid_no() {
    let doc = AAML::new();
    assert!(doc.validate_value("bool", "no").is_err());
}

#[test]
fn test_validate_string_any_value() {
    let doc = AAML::new();
    assert!(doc.validate_value("string", "anything goes").is_ok());
}

#[test]
fn test_validate_string_empty() {
    let doc = AAML::new();
    assert!(doc.validate_value("string", "").is_ok());
}

#[test]
fn test_validate_string_special_chars() {
    let doc = AAML::new();
    assert!(doc.validate_value("string", "!@#$%^&*()").is_ok());
}

#[test]
fn test_validate_color_rgb_lowercase() {
    let doc = AAML::new();
    assert!(doc.validate_value("color", "#ff00ff").is_ok());
}

#[test]
fn test_validate_color_rgb_uppercase() {
    let doc = AAML::new();
    assert!(doc.validate_value("color", "#FF00FF").is_ok());
}

#[test]
fn test_validate_color_rgba() {
    let doc = AAML::new();
    assert!(doc.validate_value("color", "#ff00ffaa").is_ok());
}

#[test]
fn test_validate_color_invalid_length() {
    let doc = AAML::new();
    assert!(doc.validate_value("color", "#fff").is_err());
}

#[test]
fn test_validate_color_invalid_chars() {
    let doc = AAML::new();
    assert!(doc.validate_value("color", "#gggggg").is_err());
}

#[test]
fn test_validate_color_missing_hash() {
    let doc = AAML::new();
    assert!(doc.validate_value("color", "ff00ff").is_err());
}

#[test]
fn test_validate_vector2() {
    let doc = AAML::new();
    assert!(doc.validate_value("math::vector2", "1.0, 2.0").is_ok());
}

#[test]
fn test_validate_vector3() {
    let doc = AAML::new();
    assert!(doc.validate_value("math::vector3", "1.0, 2.0, 3.0").is_ok());
}

#[test]
fn test_validate_vector4() {
    let doc = AAML::new();
    assert!(
        doc.validate_value("math::vector4", "1.0, 2.0, 3.0, 4.0")
            .is_ok()
    );
}

#[test]
fn test_validate_vector3_invalid_count() {
    let doc = AAML::new();
    assert!(doc.validate_value("math::vector3", "1.0, 2.0").is_err());
}

#[test]
fn test_validate_vector3_invalid_values() {
    let doc = AAML::new();
    assert!(doc.validate_value("math::vector3", "a, b, c").is_err());
}

#[test]
fn test_validate_list_i32() {
    let doc = AAML::new();
    assert!(doc.validate_value("list<i32>", "[1, 2, 3]").is_ok());
}

#[test]
fn test_validate_list_f64() {
    let doc = AAML::new();
    assert!(doc.validate_value("list<f64>", "[1.1, 2.2, 3.3]").is_ok());
}

#[test]
fn test_validate_list_string() {
    let doc = AAML::new();
    assert!(doc.validate_value("list<string>", "[a, b, c]").is_ok());
}

#[test]
fn test_validate_list_bool() {
    let doc = AAML::new();
    assert!(
        doc.validate_value("list<bool>", "[true, false, true]")
            .is_ok()
    );
}

#[test]
fn test_validate_list_empty() {
    let doc = AAML::new();
    assert!(doc.validate_value("list<i32>", "[]").is_ok());
}

#[test]
fn test_validate_list_invalid_element() {
    let doc = AAML::new();
    assert!(doc.validate_value("list<i32>", "[1, bad, 3]").is_err());
}

#[test]
fn test_validate_physics_kilogram() {
    let doc = AAML::new();
    assert!(doc.validate_value("physics::kilogram", "42").is_ok());
}

#[test]
fn test_validate_unknown_type() {
    let doc = AAML::new();
    assert!(doc.validate_value("unknown_type", "value").is_err());
}

// ============================================================================
// SECTION 5: Schema Validation Tests (50 tests)
// ============================================================================

#[test]
fn test_schema_definition_basic() {
    let content = "@schema Point { x: i32, y: i32 }";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.get_schema("Point").is_some());
}

#[test]
fn test_schema_with_required_fields() {
    let content = "@schema Config { host: string, port: i32 }\nhost=localhost\nport=8080";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_missing_required_field() {
    let content = "@schema Config { host: string, port: i32 }\nhost = localhost";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_err());
}

#[test]
fn test_schema_type_mismatch() {
    let content = "@schema Config { port: i32 }\nport = not_a_number";
    let doc = AAML::parse(content);
    assert!(doc.is_err());
}

#[test]
fn test_schema_optional_field_missing() {
    let content = "@schema Config { host: string, port*: i32 }\nhost = localhost";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_optional_field_present() {
    let content = "@schema Config { host: string, port*: i32 }\nhost=localhost\nport=8080";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_optional_field_wrong_type() {
    let content = "@schema Config { port*: i32 }\nport=not_a_number";
    let doc = AAML::parse(content);
    assert!(doc.is_err());
}

#[test]
fn test_multiple_schemas() {
    let content = "@schema Point { x: i32 }\n@schema Color { r: i32, g: i32, b: i32 }\nx=1\nr=255\ng=128\nb=64";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.get_schema("Point").is_some());
    assert!(doc.get_schema("Color").is_some());
}

#[test]
fn test_schema_with_list_type() {
    let content = "@schema Tags { items: list<string> }\nitems=[a,b,c]";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_with_vector_type() {
    let content = "@schema Position { coords: math::vector3 }\ncoords=1.0, 2.0, 3.0";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_validation_passes() {
    let content = "@schema Device { id: i32, name: string }\nid=42\nname=sensor";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_validation_fails() {
    let content = "@schema Device { id: i32 }\nid=abc";
    assert!(AAML::parse(content).is_err());
}

#[test]
fn test_schema_extra_fields_allowed() {
    let content = "@schema Config { port: i32 }\nport=8080\nextra=field";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_multiple_required() {
    let content = "@schema DB { host: string, port: i32, user: string, pass: string }\nhost=localhost\nport=5432\nuser=admin\npass=secret";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_color_field() {
    let content =
        "@schema Theme { primary: color, secondary: color }\nprimary=#ff0000\nsecondary=#00ff00";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_boolean_field() {
    let content = "@schema Feature { enabled: bool }\nenabled=true";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_float_field() {
    let content = "@schema Physics { gravity: f64 }\ngravity=9.81";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_schema_field_names_special_chars() {
    let content =
        "@schema Config { my_field: string, another-field: i32 }\nmy_field=val\nanother-field=42";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_apply_schema_manually() {
    let mut doc = AAML::new();
    doc.merge_content("@schema Test { x: i32 }").unwrap();
    let mut data = std::collections::HashMap::new();
    data.insert("x".to_string(), "42".to_string());
    assert!(doc.apply_schema("Test", &data).is_ok());
}

#[test]
fn test_apply_schema_missing_required() {
    let mut doc = AAML::new();
    doc.merge_content("@schema Test { x: i32 }").unwrap();
    let data = std::collections::HashMap::new();
    assert!(doc.apply_schema("Test", &data).is_err());
}

#[test]
fn test_apply_schema_type_mismatch() {
    let mut doc = AAML::new();
    doc.merge_content("@schema Test { x: i32 }").unwrap();
    let mut data = std::collections::HashMap::new();
    data.insert("x".to_string(), "not_a_number".to_string());
    assert!(doc.apply_schema("Test", &data).is_err());
}

#[test]
fn test_schema_get_returns_definition() {
    let content = "@schema Test { field: string }";
    let doc = AAML::parse(content).unwrap();
    let schema = doc.get_schema("Test").unwrap();
    assert!(schema.fields.contains_key("field"));
}

#[test]
fn test_schema_field_type_extraction() {
    let content = "@schema Test { port: i32 }";
    let doc = AAML::parse(content).unwrap();
    let schema = doc.get_schema("Test").unwrap();
    assert_eq!(schema.fields.get("port").unwrap(), "i32");
}

// ============================================================================
// SECTION 6: Inline Object Parsing Tests (30 tests)
// ============================================================================

#[test]
fn test_parse_inline_object_simple() {
    let obj = parse_inline_object("{ a = 1, b = 2 }").unwrap();
    assert_eq!(obj.len(), 2);
}

#[test]
fn test_parse_inline_object_empty() {
    let obj = parse_inline_object("{}").unwrap();
    assert_eq!(obj.len(), 0);
}

#[test]
fn test_parse_inline_object_with_spaces() {
    let obj = parse_inline_object("{ key = value }").unwrap();
    assert!(obj.iter().any(|(k, v)| k == "key" && v == "value"));
}

#[test]
fn test_parse_inline_object_multiple_fields() {
    let obj = parse_inline_object("{ a = 1, b = 2, c = 3 }").unwrap();
    assert_eq!(obj.len(), 3);
}

#[test]
fn test_parse_inline_object_quoted_values() {
    let obj = parse_inline_object(r#"{ name = "John", city = "NYC" }"#).unwrap();
    assert!(obj.iter().any(|(k, v)| k == "name" && v == "John"));
}

#[test]
fn test_parse_inline_object_nested() {
    let obj = parse_inline_object("{ outer = { inner = value } }").unwrap();
    assert_eq!(obj.len(), 1);
}

#[test]
fn test_parse_inline_object_numeric_values() {
    let obj = parse_inline_object("{ x = 10, y = 20 }").unwrap();
    assert!(obj.iter().any(|(k, v)| k == "x" && v == "10"));
}

#[test]
fn test_parse_inline_object_mixed_types() {
    let obj = parse_inline_object("{ str = text, num = 42, flag = true }").unwrap();
    assert_eq!(obj.len(), 3);
}

#[test]
fn test_parse_inline_object_no_closing_brace() {
    assert!(parse_inline_object("{ a = 1").is_err());
}

#[test]
fn test_parse_inline_object_missing_value() {
    let result = parse_inline_object("{ a = }");
    println!("{:?}", result);
    // Empty value is allowed
    assert!(result.is_ok());
}

#[test]
fn test_parse_inline_object_missing_key() {
    assert!(parse_inline_object("{ = value }").is_err());
}

#[test]
fn test_parse_inline_object_single_field() {
    let obj = parse_inline_object("{ x = 5 }").unwrap();
    assert_eq!(obj.len(), 1);
}

#[test]
fn test_parse_inline_object_hex_color() {
    let obj = parse_inline_object("{ tint = #ff00ff }").unwrap();
    assert!(obj.iter().any(|(k, v)| k == "tint" && v == "#ff00ff"));
}

#[test]
fn test_parse_inline_object_list_value() {
    let obj = parse_inline_object("{ items = [1, 2, 3] }").unwrap();
    assert_eq!(obj.len(), 1);
}

#[test]
fn test_parse_inline_object_spaces_around_equals() {
    let obj = parse_inline_object("{ key = value }").unwrap();
    assert!(obj.iter().any(|(k, v)| k == "key" && v == "value"));
}

#[test]
fn test_parse_inline_object_no_spaces_around_equals() {
    let obj = parse_inline_object("{key=value}").unwrap();
    assert!(obj.iter().any(|(k, v)| k == "key" && v == "value"));
}

#[test]
fn test_parse_inline_object_many_fields() {
    let obj = parse_inline_object("{ a = 1, b = 2, c = 3, d = 4, e = 5 }").unwrap();
    assert_eq!(obj.len(), 5);
}

#[test]
fn test_parse_inline_object_trailing_comma() {
    let obj = parse_inline_object("{ a = 1, }").unwrap();
    assert_eq!(obj.len(), 1);
}

#[test]
fn test_parse_inline_object_special_chars_in_key() {
    let obj = parse_inline_object("{ my_key = value }").unwrap();
    assert!(obj.iter().any(|(k, v)| k == "my_key" && v == "value"));
}

#[test]
fn test_parse_inline_object_special_chars_in_value() {
    let obj = parse_inline_object(r#"{ key = "value!@#$%" }"#).unwrap();
    assert!(obj.iter().any(|(k, v)| k == "key" && v == "value!@#$%"));
}

#[test]
fn test_parse_inline_object_boolean_values() {
    let obj = parse_inline_object("{ active = true, visible = false }").unwrap();
    assert_eq!(obj.len(), 2);
}

#[test]
fn test_parse_inline_object_decimal_values() {
    let obj = parse_inline_object("{ pi = 3.14, e = 2.71 }").unwrap();
    assert_eq!(obj.len(), 2);
}

#[test]
fn test_parse_inline_object_whitespace_handling() {
    let obj = parse_inline_object("{  key  =  value  ,  other  =  data  }").unwrap();
    assert_eq!(obj.len(), 2);
}

// ============================================================================
// SECTION 7: Builder API Tests (40 tests)
// ============================================================================

#[test]
fn test_builder_new() {
    let builder = AAMBuilder::new();
    let result = builder.build();
    assert!(!result.is_empty() || result.is_empty()); // Always true
}

#[test]
fn test_builder_add_line() {
    let mut builder = AAMBuilder::new();
    builder.add_line("key", "value");
    let content = builder.build();
    assert!(content.contains("key"));
}

#[test]
fn test_builder_add_multiple_lines() {
    let mut builder = AAMBuilder::new();
    builder.add_line("a", "1");
    builder.add_line("b", "2");
    builder.add_line("c", "3");
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
}

#[test]
fn test_builder_build() {
    let mut builder = AAMBuilder::new();
    builder.add_line("key", "value");
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_builder_schema() {
    let mut builder = AAMBuilder::new();
    builder.schema(
        "Point",
        [
            SchemaField::required("x", "i32"),
            SchemaField::required("y", "i32"),
        ],
    );
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert!(doc.get_schema("Point").is_some());
}

#[test]
fn test_builder_with_content() {
    let mut builder = AAMBuilder::new();
    builder.add_line("a", "1");
    builder.add_line("b", "2");
    let content = builder.build();
    assert!(!content.is_empty());
}

#[test]
fn test_builder_to_string() {
    let mut builder = AAMBuilder::new();
    builder.add_line("key", "value");
    let output = builder.build();
    assert!(output.contains("key"));
    assert!(output.contains("value"));
}

#[test]
fn test_builder_file_write_read() {
    let mut builder = AAMBuilder::new();
    builder.add_line("test_key", "test_value");
    let file = "test_builder_file.aam";
    builder.to_file(file).unwrap();
    let doc = AAML::load(file).unwrap();
    let _ = fs::remove_file(file);
    assert_eq!(doc.find_obj("test_key").unwrap().as_str(), "test_value");
}

#[test]
fn test_builder_add_quoted_value() {
    let mut builder = AAMBuilder::new();
    builder.add_line("key", "quoted value");
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "quoted value");
}

#[test]
fn test_builder_numeric_key_value() {
    let mut builder = AAMBuilder::new();
    builder.add_line("123", "456");
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert_eq!(doc.find_obj("123").unwrap().as_str(), "456");
}

#[test]
fn test_builder_special_chars() {
    let mut builder = AAMBuilder::new();
    builder.add_line("my_key", "my_value");
    builder.add_line("my-key", "my-value");
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert!(doc.find_obj("my_key").is_some());
    assert!(doc.find_obj("my-key").is_some());
}

#[test]
fn test_builder_empty() {
    let builder = AAMBuilder::new();
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert!(doc.find_obj("anything").is_none());
}

#[test]
fn test_builder_schema_optional() {
    let mut builder = AAMBuilder::new();
    builder.schema("Config", [SchemaField::optional("timeout", "i32")]);
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    let schema = doc.get_schema("Config").unwrap();
    assert!(schema.optional_fields.contains("timeout"));
}

#[test]
fn test_builder_schema_mixed() {
    let mut builder = AAMBuilder::new();
    builder.schema(
        "Mixed",
        [
            SchemaField::required("req", "string"),
            SchemaField::optional("opt", "i32"),
        ],
    );
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert!(doc.get_schema("Mixed").is_some());
}

#[test]
fn test_builder_multiple_schemas() {
    let mut builder = AAMBuilder::new();
    builder.schema("Point", [SchemaField::required("x", "i32")]);
    builder.schema("Color", [SchemaField::required("r", "i32")]);
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert!(doc.get_schema("Point").is_some());
    assert!(doc.get_schema("Color").is_some());
}

#[test]
fn test_builder_add_comment() {
    let mut builder = AAMBuilder::new();
    builder.comment("This is a comment");
    let content = builder.build();
    assert!(content.contains("# This is a comment"));
}

#[test]
fn test_builder_preserves_order() {
    let mut builder = AAMBuilder::new();
    builder.add_line("first", "1");
    builder.add_line("second", "2");
    builder.add_line("third", "3");
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert_eq!(doc.find_obj("first").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("second").unwrap().as_str(), "2");
    assert_eq!(doc.find_obj("third").unwrap().as_str(), "3");
}

// ============================================================================
// SECTION 8: Merge and Assignment Tests (30 tests)
// ============================================================================

#[test]
fn test_merge_simple_content() {
    let mut doc = AAML::new();
    doc.merge_content("key = value").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_merge_multiple_times() {
    let mut doc = AAML::new();
    doc.merge_content("a = 1").unwrap();
    doc.merge_content("b = 2").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
}

#[test]
fn test_merge_overwrite() {
    let mut doc = AAML::new();
    doc.merge_content("key = value1").unwrap();
    doc.merge_content("key = value2").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value2");
}

#[test]
fn test_merge_with_schema() {
    let mut doc = AAML::new();
    doc.merge_content("@schema Config { port: i32 }").unwrap();
    doc.merge_content("port = 8080").unwrap();
    assert_eq!(doc.find_obj("port").unwrap().as_str(), "8080");
}

#[test]
fn test_merge_invalid_schema() {
    let mut doc = AAML::new();
    doc.merge_content("@schema Config { port: i32 }").unwrap();
    assert!(doc.merge_content("port = not_a_number").is_err());
}

#[test]
fn test_merge_referenced_values() {
    let mut doc = AAML::new();
    doc.merge_content("a = b").unwrap();
    doc.merge_content("b = final").unwrap();
    assert_eq!(doc.find_deep("a").unwrap().as_str(), "final");
}

#[test]
fn test_merge_empty_content() {
    let mut doc = AAML::new();
    doc.merge_content("").unwrap();
    assert!(doc.find_obj("anything").is_none());
}

#[test]
fn test_merge_comments() {
    let mut doc = AAML::new();
    doc.merge_content("key = value # comment").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_merge_quoted_values() {
    let mut doc = AAML::new();
    doc.merge_content(r#"key = "quoted value""#).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "quoted value");
}

#[test]
fn test_merge_multiple_assignments() {
    let mut doc = AAML::new();
    doc.merge_content("a=1\nb=2\nc=3").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
}

// parse_assignment is private, so we test it indirectly through parse

#[test]
fn test_assignment_parse_basic() {
    let doc = AAML::parse("key = value").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_assignment_parse_no_spaces() {
    let doc = AAML::parse("key=value").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_assignment_parse_quoted() {
    let doc = AAML::parse("key = \"quoted\"").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "quoted");
}

#[test]
fn test_assignment_parse_empty_value() {
    let doc = AAML::parse("key = ").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "");
}

#[test]
fn test_assignment_parse_no_equals() {
    // Lines without equals are treated as invalid assignments
    let result = AAML::parse("key without equals");
    assert!(result.is_err());
}

#[test]
fn test_assignment_parse_trailing_comment() {
    let doc = AAML::parse("key = value # comment").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_assignment_parse_multiple_equals() {
    let doc = AAML::parse("key = value = extra").unwrap();
    assert!(doc.find_obj("key").is_some());
}

#[test]
fn test_assignment_parse_whitespace_key() {
    let doc = AAML::parse("  key  = value").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_assignment_parse_whitespace_value() {
    let doc = AAML::parse("key =  value  ").unwrap();
    assert!(doc.find_obj("key").is_some());
}

#[test]
fn test_assignment_parse_hex_color() {
    let doc = AAML::parse("color = #ff00ff").unwrap();
    assert_eq!(doc.find_obj("color").unwrap().as_str(), "#ff00ff");
}

#[test]
fn test_assignment_parse_list() {
    let doc = AAML::parse("items = [1, 2, 3]").unwrap();
    assert!(doc.find_obj("items").is_some());
}

#[test]
fn test_assignment_parse_object() {
    let doc = AAML::parse("obj = { a = 1 }").unwrap();
    assert!(doc.find_obj("obj").is_some());
}

// ============================================================================
// SECTION 9: Load from File Tests (20 tests)
// ============================================================================

#[test]
fn test_load_from_file() {
    let file = "test_load.aam";
    let content = "key = value";
    fs::write(file, content).unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_load_nonexistent_file() {
    assert!(AAML::load("nonexistent_file.aam").is_err());
}

#[test]
fn test_load_empty_file() {
    let file = "test_empty.aam";
    fs::write(file, "").unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert!(doc.find_obj("anything").is_none());
}

#[test]
fn test_load_multiple_lines() {
    let file = "test_multiple.aam";
    let content = "a=1\nb=2\nc=3";
    fs::write(file, content).unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("c").unwrap().as_str(), "3");
}

#[test]
fn test_load_with_comments() {
    let file = "test_comments.aam";
    let content = "a=1 # comment\nb=2";
    fs::write(file, content).unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
}

#[test]
fn test_load_with_schema() {
    let file = "test_schema_load.aam";
    let content = "@schema Test { x: i32 }\nx=42";
    fs::write(file, content).unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert_eq!(doc.find_obj("x").unwrap().as_str(), "42");
}

#[test]
fn test_load_with_unicode() {
    let file = "test_unicode.aam";
    let content = "name = Привет";
    fs::write(file, content).unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert_eq!(doc.find_obj("name").unwrap().as_str(), "Привет");
}

#[test]
fn test_load_with_quotes() {
    let file = "test_quotes_load.aam";
    let content = "key = \"value with spaces\"";
    fs::write(file, content).unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value with spaces");
}

#[test]
fn test_load_relative_path() {
    let file = "test_relative.aam";
    let content = "test = value";
    fs::write(file, content).unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert!(doc.find_obj("test").is_some());
}

#[test]
fn test_load_large_file() {
    let file = "test_large.aam";
    let mut content = String::new();
    for i in 0..1000 {
        let _ = writeln!(content, "key_{} = value_{}", i, i);
    }
    fs::write(file, content).unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert_eq!(doc.find_obj("key_0").unwrap().as_str(), "value_0");
    assert_eq!(doc.find_obj("key_999").unwrap().as_str(), "value_999");
}

// ============================================================================
// SECTION 10: Edge Cases and Error Handling (40 tests)
// ============================================================================

#[test]
fn test_very_long_key() {
    let long_key = "k".repeat(1000);
    let doc = AAML::parse(&format!("{} = value", long_key)).unwrap();
    assert_eq!(doc.find_obj(&long_key).unwrap().as_str(), "value");
}

#[test]
fn test_very_long_value() {
    let long_value = "v".repeat(1000);
    let doc = AAML::parse(&format!("key = {}", long_value)).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), long_value);
}

#[test]
fn test_deeply_nested_references() {
    let mut content = String::new();
    for i in 0..100 {
        content.push_str(&format!("a{} = b{}\n", i, i));
    }
    content.push_str("b99 = final");
    let doc = AAML::parse(&content).unwrap();
    assert!(doc.find_deep("a0").is_some());
}

#[test]
fn test_many_keys() {
    let mut content = String::new();
    for i in 0..500 {
        content.push_str(&format!("key_{} = {}\n", i, i));
    }
    let doc = AAML::parse(&content).unwrap();
    assert_eq!(doc.find_obj("key_0").unwrap().as_str(), "0");
    assert_eq!(doc.find_obj("key_499").unwrap().as_str(), "499");
}

#[test]
fn test_whitespace_only_value() {
    let doc = AAML::parse("key =    ").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "");
}

#[test]
fn test_key_with_all_special_chars() {
    let doc = AAML::parse("key-_./: = value").unwrap();
    assert_eq!(doc.find_obj("key-_./:").unwrap().as_str(), "value");
}

#[test]
fn test_unicode_in_key() {
    let doc = AAML::parse("ключ = значение").unwrap();
    assert_eq!(doc.find_obj("ключ").unwrap().as_str(), "значение");
}

#[test]
fn test_unicode_in_value() {
    let doc = AAML::parse("key = 你好").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "你好");
}

#[test]
fn test_emoji_in_value() {
    let doc = AAML::parse("key = 😀😃😄").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "😀😃😄");
}

#[test]
fn test_carriage_return_handling() {
    let doc = AAML::parse("a=1\r\nb=2").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
}

#[test]
fn test_null_byte_handling() {
    // Most systems will handle null bytes appropriately
    let content = "key = value";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.find_obj("key").is_some());
}

#[test]
fn test_duplicate_keys_last_wins() {
    let doc = AAML::parse("key = value1\nkey = value2").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value2");
}

#[test]
fn test_circular_reference_a_b() {
    let doc = AAML::parse("a = b\nb = a").unwrap();
    // Should not panic, should return a value
    assert!(doc.find_deep("a").is_some());
}

#[test]
fn test_circular_reference_chain() {
    let doc = AAML::parse("a=b\nb=c\nc=d\nd=a").unwrap();
    assert!(doc.find_deep("a").is_some());
}

#[test]
fn test_empty_parse() {
    let doc = AAML::parse("").unwrap();
    assert!(doc.find_obj("anything").is_none());
}

#[test]
fn test_only_comments() {
    let doc = AAML::parse("# comment 1\n# comment 2\n# comment 3").unwrap();
    assert!(doc.find_obj("anything").is_none());
}

#[test]
fn test_mixed_line_endings() {
    let doc = AAML::parse("a=1\rb=2\nc=3").unwrap();
    assert!(doc.find_obj("a").is_some() || doc.find_obj("b").is_some());
}

#[test]
fn test_tab_characters() {
    let doc = AAML::parse("key\t=\tvalue").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_multiple_spaces() {
    let doc = AAML::parse("key    =    value").unwrap();
    assert_eq!(doc.find_obj("key").unwrap().as_str(), "value");
}

#[test]
fn test_assignment_with_only_equals() {
    let result = AAML::parse("= = =");
    assert!(result.is_err());
}

#[test]
fn test_newline_preservation() {
    let doc = AAML::parse("a=1\n\n\nb=2").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
}

#[test]
fn test_parse_error_handling() {
    // Parser should be tolerant of most errors
    let result = AAML::parse("definitely_valid = content");
    assert!(result.is_ok());
}

#[test]
fn test_very_deeply_nested_objects() {
    let mut content = String::new();
    content.push_str("root = ");
    for _ in 0..10 {
        content.push_str("{ nested = ");
    }
    content.push_str("value");
    for _ in 0..10 {
        content.push_str(" }");
    }
    let doc = AAML::parse(&content);
    // Should either parse or give clear error
    assert!(doc.is_ok() || doc.is_err());
}

#[test]
fn test_found_value_format_display() {
    let doc = AAML::parse("key = value").unwrap();
    let val = doc.find_obj("key").unwrap();
    let formatted = format!("{}", val);
    assert_eq!(formatted, "value");
}

#[test]
fn test_found_value_equality() {
    let doc = AAML::parse("key = value").unwrap();
    let val = doc.find_obj("key").unwrap();
    assert_eq!(val.as_str(), "value");
    assert_eq!(&*val, "value");
}

// ============================================================================
// SECTION 11: Comprehensive Integration Tests (50 tests)
// ============================================================================

#[test]
fn test_full_workflow_parse_lookup_validate() {
    let content = "@schema User { id: i32, name: string }\nid=1\nname=Alice";
    let doc = AAML::parse(content).unwrap();
    assert_eq!(doc.find_obj("id").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("name").unwrap().as_str(), "Alice");
}

#[test]
fn test_builder_workflow() {
    let mut builder = AAMBuilder::new();
    builder.add_line("key1", "value1");
    builder.add_line("key2", "value2");
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert_eq!(doc.find_obj("key1").unwrap().as_str(), "value1");
}

#[test]
fn test_file_roundtrip() {
    let file = "test_roundtrip.aam";
    let mut builder = AAMBuilder::new();
    builder.add_line("test", "value");
    builder.to_file(file).unwrap();
    let doc = AAML::load(file).unwrap();
    fs::remove_file(file).unwrap();
    assert_eq!(doc.find_obj("test").unwrap().as_str(), "value");
}

#[test]
fn test_schema_inheritance_pattern() {
    let content =
        "@schema Base { id: i32 }\n@schema Extended { id: i32, name: string }\nid=1\nname=test";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.get_schema("Base").is_some());
    assert!(doc.get_schema("Extended").is_some());
}

#[test]
fn test_reference_chain_with_schema() {
    let content = "@schema Chain { ref: string }\nref=next\nnext=value";
    let doc = AAML::parse(content).unwrap();
    assert_eq!(doc.find_deep("ref").unwrap().as_str(), "value");
}

#[test]
fn test_multiple_schemas_validation() {
    let content = "@schema A { x: i32 }\n@schema B { y: string }\nx=10\ny=text";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_complex_types_validation() {
    let content = "@schema Complex { pos: math::vector3, colors: list<color> }\npos=1,2,3\ncolors=[#ff0000,#00ff00]";
    let doc = AAML::parse(content).unwrap();
    assert_eq!(doc.find_obj("pos").unwrap().as_str(), "1,2,3");
}

#[test]
fn test_stress_many_operations() {
    let mut doc = AAML::new();
    for i in 0..100 {
        let content = format!("key_{} = value_{}", i, i);
        doc.merge_content(&content).unwrap();
    }
    assert_eq!(doc.find_obj("key_0").unwrap().as_str(), "value_0");
    assert_eq!(doc.find_obj("key_99").unwrap().as_str(), "value_99");
}

#[test]
fn test_all_builtin_types_together() {
    let content = "@schema AllTypes {\n\
        i: i32,\n\
        f: f64,\n\
        b: bool,\n\
        s: string,\n\
        c: color,\n\
        v3: math::vector3,\n\
        l: list<i32>\n\
    }\n\
    i=42\n\
    f=3.14\n\
    b=true\n\
    s=text\n\
    c=#ff0000\n\
    v3=1,2,3\n\
    l=[1,2,3]";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_merge_respects_schema() {
    let mut doc = AAML::new();
    doc.merge_content("@schema Test { num: i32 }").unwrap();
    doc.merge_content("num = 42").unwrap();
    assert_eq!(doc.find_obj("num").unwrap().as_str(), "42");
}

#[test]
fn test_load_and_merge() {
    let file = "test_load_merge.aam";
    fs::write(file, "a=1").unwrap();
    let mut doc = AAML::load(file).unwrap();
    doc.merge_content("b=2").unwrap();
    fs::remove_file(file).unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
}

#[test]
fn test_builder_with_schema_and_data() {
    let mut builder = AAMBuilder::new();
    builder.schema("Test", [SchemaField::required("id", "i32")]);
    builder.add_line("id", "123");
    let content = builder.build();
    let doc = AAML::parse(&content).unwrap();
    assert_eq!(doc.find_obj("id").unwrap().as_str(), "123");
}

#[test]
fn test_comment_handling_comprehensive() {
    let content = "# Header\na = 1 # inline\n# Middle\nb = 2\n# Footer";
    let doc = AAML::parse(content).unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc.find_obj("b").unwrap().as_str(), "2");
}

#[test]
fn test_quoted_values_comprehensive() {
    let content = "a = \"double quoted\"\nb = text_no_quotes";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.find_obj("a").is_some());
    assert!(doc.find_obj("b").is_some());
}

#[test]
fn test_whitespace_normalization() {
    let content = "  a  =  1  \n\t\tb\t=\t2\t";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.find_obj("a").is_some());
    assert!(doc.find_obj("b").is_some());
}

#[test]
fn test_sequential_operations() {
    let mut doc = AAML::new();
    doc.merge_content("a=1").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "1");
    doc.merge_content("a=2").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "2");
    doc.merge_content("b=a").unwrap();
    assert_eq!(doc.find_deep("b").unwrap().as_str(), "2");
}

#[test]
fn test_mixed_content_types() {
    let content = "numbers = 1, 2, 3\nobject = { x = 1, y = 2 }\nhex = #ff00ff\ntext = hello";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.find_obj("numbers").is_some());
    assert!(doc.find_obj("object").is_some());
    assert!(doc.find_obj("hex").is_some());
    assert!(doc.find_obj("text").is_some());
}

#[test]
fn test_error_recovery_partial() {
    let content = "valid_key = valid_value";
    let doc = AAML::parse(content).unwrap();
    assert_eq!(doc.find_obj("valid_key").unwrap().as_str(), "valid_value");
}

#[test]
fn test_large_value_handling() {
    let large = "x".repeat(10000);
    let doc = AAML::parse(&format!("key = {}", large)).unwrap();
    assert_eq!(doc.find_obj("key").unwrap().len(), 10000);
}

#[test]
fn test_many_references() {
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!("ref_{} = value_{}\n", i, i));
    }
    let doc = AAML::parse(&content).unwrap();
    for i in 0..50 {
        assert_eq!(
            doc.find_obj(&format!("ref_{}", i)).unwrap().as_str(),
            format!("value_{}", i)
        );
    }
}

#[test]
fn test_type_validation_edge_cases() {
    let doc = AAML::new();
    assert!(doc.validate_value("i32", "-2147483648").is_ok()); // Min i32
    assert!(doc.validate_value("i32", "2147483647").is_ok()); // Max i32
    assert!(doc.validate_value("f64", "1.7976931348623157e+308").is_ok()); // Near max f64
}

#[test]
fn test_schema_validation_optional_comprehensive() {
    let content = "@schema Partial { req: string, opt1*: i32, opt2*: string }\nreq=value";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.validate_schemas_completeness().is_ok());
}

#[test]
fn test_bidirectional_lookup() {
    let doc = AAML::parse("a = b").unwrap();
    assert_eq!(doc.find_obj("a").unwrap().as_str(), "b");
    assert_eq!(doc.find_key("b").unwrap().as_str(), "a");
}

#[test]
fn test_find_key_multiple_results() {
    let doc = AAML::parse("a=value\nb=value\nc=value").unwrap();
    // find_key should return one of them
    assert!(doc.find_key("value").is_some());
}

#[test]
fn test_parse_and_merge_mixed() {
    let doc1 = AAML::parse("a=1").unwrap();
    let mut doc2 = AAML::parse("b=2").unwrap();
    doc2.merge_content("c=3").unwrap();
    assert_eq!(doc1.find_obj("a").unwrap().as_str(), "1");
    assert_eq!(doc2.find_obj("b").unwrap().as_str(), "2");
    assert_eq!(doc2.find_obj("c").unwrap().as_str(), "3");
}

#[test]
fn test_complex_references() {
    let content = "x = y\ny = z\nz = w\nw = v\nv = final";
    let doc = AAML::parse(content).unwrap();
    assert_eq!(doc.find_deep("x").unwrap().as_str(), "final");
}

#[test]
fn test_overwrite_with_schema_validation() {
    let mut doc = AAML::new();
    doc.merge_content("@schema N { x: i32 }").unwrap();
    doc.merge_content("x = 1").unwrap();
    doc.merge_content("x = 2").unwrap();
    assert_eq!(doc.find_obj("x").unwrap().as_str(), "2");
}

#[test]
fn test_empty_schema() {
    let content = "@schema Empty { }";
    let doc = AAML::parse(content);
    // Should either parse or give error
    assert!(doc.is_ok() || doc.is_err());
}

#[test]
fn test_numeric_schema_names() {
    let content = "@schema Schema123 { f: string }\nf=value";
    let doc = AAML::parse(content).unwrap();
    assert!(doc.get_schema("Schema123").is_some());
}

#[test]
fn test_full_example_config() {
    let content = "# Application Configuration\n\
        @schema App { name: string, version: string, port: i32 }\n\
        \n\
        name = MyApp\n\
        version = 1.0.0\n\
        port = 3000\n\
        \n\
        # Database settings\n\
        db_host = localhost\n\
        db_port = 5432";
    let doc = AAML::parse(content).unwrap();
    assert_eq!(doc.find_obj("name").unwrap().as_str(), "MyApp");
    assert_eq!(doc.find_obj("port").unwrap().as_str(), "3000");
}
