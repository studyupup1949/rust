use std::collections::HashMap;

use adot::config::Variable;
use adot::template::render;

fn make_vars() -> HashMap<String, Variable> {
    let mut vars = HashMap::new();

    let mut git = HashMap::new();
    git.insert(
        "email".to_string(),
        Variable::Value("user@test.com".to_string()),
    );
    vars.insert("git".to_string(), Variable::Nested(git));

    let mut colors = HashMap::new();
    colors.insert(
        "foreground".to_string(),
        Variable::Value("#AABBCC".to_string()),
    );
    colors.insert(
        "background".to_string(),
        Variable::Value("#000000".to_string()),
    );
    vars.insert("colors".to_string(), Variable::Nested(colors));

    vars.insert("editor".to_string(), Variable::Value("nvim".to_string()));

    vars
}

//////////////////////////////////////////////////////////////////////
// TEST TEMPLATE VARIABLE REPLACEMENT

#[test]
fn replace_simple_variable() {
    let vars = make_vars();
    let result = render("editor = {{@@ editor @@}}\n", &vars, "test").unwrap();
    assert_eq!(result, "editor = nvim\n");
}

#[test]
fn replace_nested_variable() {
    let vars = make_vars();
    let result = render("email = {{@@ git.email @@}}\n", &vars, "test").unwrap();
    assert_eq!(result, "email = user@test.com\n");
}

#[test]
fn replace_multiple_variables_on_one_line() {
    let vars = make_vars();
    let input = "fg={{@@ colors.foreground @@}} bg={{@@ colors.background @@}}\n";
    let result = render(input, &vars, "test").unwrap();
    assert_eq!(result, "fg=#AABBCC bg=#000000\n");
}

#[test]
fn undefined_variable_fails() {
    let vars = make_vars();
    let err = render("{{@@ nonexistent @@}}", &vars, "test").unwrap_err();
    assert!(err.contains("undefined variable"), "got: {err}");
}

#[test]
fn undefined_nested_variable_fails() {
    let vars = make_vars();
    let err = render("{{@@ git.name @@}}", &vars, "test").unwrap_err();
    assert!(err.contains("undefined variable"), "got: {err}");
}

#[test]
fn unclosed_variable_fails() {
    let vars = make_vars();
    let err = render("{{@@ editor", &vars, "test").unwrap_err();
    assert!(err.contains("unclosed"), "got: {err}");
}

//////////////////////////////////////////////////////////////////////
// TEST TEMPLATE CONDITIONALS

#[test]
fn conditional_profile_match_includes_block() {
    let vars = make_vars();
    let input = "before\n{%@@ if profile == \"test-host\" @@%}\nincluded\n{%@@ endif @@%}\nafter\n";
    let result = render(input, &vars, "test-host").unwrap();
    assert_eq!(result, "before\nincluded\nafter\n");
}

#[test]
fn conditional_profile_no_match_excludes_block() {
    let vars = make_vars();
    let input =
        "before\n{%@@ if profile == \"other-host\" @@%}\nexcluded\n{%@@ endif @@%}\nafter\n";
    let result = render(input, &vars, "test-host").unwrap();
    assert_eq!(result, "before\nafter\n");
}

#[test]
fn conditional_unclosed_fails() {
    let vars = make_vars();
    let input = "{%@@ if profile == \"x\" @@%}\nstuff\n";
    let err = render(input, &vars, "test").unwrap_err();
    assert!(err.contains("unclosed"), "got: {err}");
}

#[test]
fn conditional_elif_first_matches() {
    let vars = make_vars();
    let input = "before\n{%@@ if profile == \"host-a\" @@%}\nbranch-a\n{%@@ elif profile == \"host-b\" @@%}\nbranch-b\n{%@@ endif @@%}\nafter\n";
    let result = render(input, &vars, "host-a").unwrap();
    assert_eq!(result, "before\nbranch-a\nafter\n");
}

#[test]
fn conditional_elif_second_matches() {
    let vars = make_vars();
    let input = "before\n{%@@ if profile == \"host-a\" @@%}\nbranch-a\n{%@@ elif profile == \"host-b\" @@%}\nbranch-b\n{%@@ endif @@%}\nafter\n";
    let result = render(input, &vars, "host-b").unwrap();
    assert_eq!(result, "before\nbranch-b\nafter\n");
}

#[test]
fn conditional_elif_none_matches() {
    let vars = make_vars();
    let input = "before\n{%@@ if profile == \"host-a\" @@%}\nbranch-a\n{%@@ elif profile == \"host-b\" @@%}\nbranch-b\n{%@@ endif @@%}\nafter\n";
    let result = render(input, &vars, "host-c").unwrap();
    assert_eq!(result, "before\nafter\n");
}

#[test]
fn conditional_elif_only_first_match_included() {
    let vars = make_vars();
    // both if and elif would match "host-a" — only the first should be included
    let input = "before\n{%@@ if profile == \"host-a\" @@%}\nfirst\n{%@@ elif profile == \"host-a\" @@%}\nsecond\n{%@@ endif @@%}\nafter\n";
    let result = render(input, &vars, "host-a").unwrap();
    assert_eq!(result, "before\nfirst\nafter\n");
}

//////////////////////////////////////////////////////////////////////
// TEST TEMPLATE COMMENTS

#[test]
fn comment_lines_stripped() {
    let vars = make_vars();
    let input = "{#@@ This is a comment @@#}\nkept\n";
    let result = render(input, &vars, "test").unwrap();
    assert_eq!(result, "kept\n");
}

//////////////////////////////////////////////////////////////////////
// TEST TEMPLATE FULL

#[test]
fn render_full_template_no_match() {
    let vars = make_vars();
    let input = r#"{#@@ Template comment @@#}
[user]
    name = Test User
    email = {{@@ git.email @@}}
{%@@ if profile == "special-host" @@%}
[http]
    proxy = http://localhost:9000
{%@@ endif @@%}
[alias]
    c = commit
"#;
    let expected = r#"[user]
    name = Test User
    email = user@test.com
[alias]
    c = commit
"#;
    let result = render(input, &vars, "test-host").unwrap();
    assert_eq!(result, expected);
}

#[test]
fn render_full_template_matching_profile() {
    let vars = make_vars();
    let input = r#"{#@@ Template comment @@#}
[user]
    email = {{@@ git.email @@}}
{%@@ if profile == "special-host" @@%}
[http]
    proxy = http://localhost:9000
{%@@ endif @@%}
[alias]
    c = commit
"#;
    let expected = r#"[user]
    email = user@test.com
[http]
    proxy = http://localhost:9000
[alias]
    c = commit
"#;
    let result = render(input, &vars, "special-host").unwrap();
    assert_eq!(result, expected);
}

//////////////////////////////////////////////////////////////////////
// TEST TEMPLATE INI-STYLE (polybar-like)

#[test]
fn render_ini_style_multiple_vars() {
    let mut vars = HashMap::new();

    let mut colors = HashMap::new();
    colors.insert(
        "background".to_string(),
        Variable::Value("#111111".to_string()),
    );
    vars.insert("colors".to_string(), Variable::Nested(colors));

    let mut polybar = HashMap::new();
    polybar.insert(
        "modules_right".to_string(),
        Variable::Value("vpn wifi lan battery".to_string()),
    );
    polybar.insert("thermal_zone".to_string(), Variable::Value("3".to_string()));
    polybar.insert(
        "wifi_interface".to_string(),
        Variable::Value("wlan0".to_string()),
    );
    polybar.insert(
        "eth_interface".to_string(),
        Variable::Value("eth0".to_string()),
    );
    vars.insert("polybar".to_string(), Variable::Nested(polybar));

    let input = r#"[colors]
background = {{@@ colors.background @@}}
background-alt = {{@@ colors.background @@}}

[bar/main]
modules-right = {{@@ polybar.modules_right @@}}

[module/temperature]
thermal-zone = {{@@ polybar.thermal_zone @@}}

[module/wifi]
interface = {{@@ polybar.wifi_interface @@}}

[module/lan]
interface = {{@@ polybar.eth_interface @@}}
"#;
    let expected = r#"[colors]
background = #111111
background-alt = #111111

[bar/main]
modules-right = vpn wifi lan battery

[module/temperature]
thermal-zone = 3

[module/wifi]
interface = wlan0

[module/lan]
interface = eth0
"#;
    let result = render(input, &vars, "test").unwrap();
    assert_eq!(result, expected);
}

//////////////////////////////////////////////////////////////////////
// TEST TEMPLATE EDGE CASES

#[test]
fn no_templates_passthrough() {
    let vars = make_vars();
    let input = "just plain text\nno templates here\n";
    let result = render(input, &vars, "test").unwrap();
    assert_eq!(result, input);
}

#[test]
fn empty_input() {
    let vars = make_vars();
    let result = render("", &vars, "test").unwrap();
    assert_eq!(result, "");
}

#[test]
fn nested_variable_not_a_map_fails() {
    let vars = make_vars();
    let err = render("{{@@ editor.sub @@}}", &vars, "test").unwrap_err();
    assert!(err.contains("not nested"), "got: {err}");
}

#[test]
fn variable_is_map_not_value_fails() {
    let vars = make_vars();
    let err = render("{{@@ colors @@}}", &vars, "test").unwrap_err();
    assert!(err.contains("nested map"), "got: {err}");
}

#[test]
fn unsupported_condition_variable_fails() {
    let vars = make_vars();
    let input = "{%@@ if hostname == \"x\" @@%}\nstuff\n{%@@ endif @@%}\n";
    let err = render(input, &vars, "test").unwrap_err();
    assert!(err.contains("unsupported condition variable"), "got: {err}");
}

#[test]
fn malformed_condition_fails() {
    let vars = make_vars();
    let input = "{%@@ if profile @@%}\nstuff\n{%@@ endif @@%}\n";
    let err = render(input, &vars, "test").unwrap_err();
    assert!(err.contains("unsupported condition"), "got: {err}");
}

#[test]
fn empty_variable_key_fails() {
    let vars = make_vars();
    let err = render("{{@@  @@}}", &vars, "test").unwrap_err();
    assert!(err.contains("empty variable key"), "got: {err}");
}
