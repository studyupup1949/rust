use std::path::PathBuf;

use adot::config::{Config, DotfileType, Variable};

#[test]
fn parse_link_children() {
    let path = PathBuf::from("tests/fixtures/config_link_children.yaml");
    let config = Config::load(Some(&path)).unwrap();

    assert_eq!(config.dotpath, PathBuf::from("dotfiles"));

    let dotfile = config.dotfiles.get("d_config").expect("d_config not found");
    assert_eq!(
        dotfile.dst,
        PathBuf::from("/tmp/adot_tests/link_children/dst/")
    );
    assert_eq!(dotfile.src, PathBuf::from("config/"));
    assert_eq!(dotfile.dtype, DotfileType::LinkChildren);

    let profile = config
        .profiles
        .get("test-host")
        .expect("test-host not found");
    assert_eq!(profile.dotfiles, vec!["d_config"]);
    assert!(profile.include.is_empty());
}

#[test]
fn parse_copy() {
    let path = PathBuf::from("tests/fixtures/config_copy.yaml");
    let config = Config::load(Some(&path)).unwrap();

    let dotfile = config.dotfiles.get("f_bashrc").expect("f_bashrc not found");
    assert_eq!(
        dotfile.dst,
        PathBuf::from("/tmp/adot_tests/copy/dst/.bashrc")
    );
    assert_eq!(dotfile.src, PathBuf::from("bashrc"));
    assert_eq!(dotfile.dtype, DotfileType::Copy);
}

#[test]
fn parse_template_with_variables() {
    let path = PathBuf::from("tests/fixtures/config_template.yaml");
    let config = Config::load(Some(&path)).unwrap();

    let dotfile = config
        .dotfiles
        .get("f_gitconfig")
        .expect("f_gitconfig not found");
    assert_eq!(dotfile.dtype, DotfileType::Template);

    let profile = config
        .profiles
        .get("test-host")
        .expect("test-host not found");

    // flat variable
    let editor = profile.variables.get("editor").expect("editor not found");
    assert_eq!(*editor, adot::config::Variable::Value("nvim".to_string()));

    // nested variable
    let git = profile.variables.get("git").expect("git not found");
    match git {
        adot::config::Variable::Nested(map) => {
            let email = map.get("email").expect("email not found");
            assert_eq!(
                *email,
                adot::config::Variable::Value("test@example.com".to_string())
            );
        }
        _ => panic!("expected nested variable for 'git'"),
    }
}

#[test]
fn parse_link_explicit() {
    let path = PathBuf::from("tests/fixtures/config_link_default.yaml");
    let config = Config::load(Some(&path)).unwrap();

    let zshrc = config.dotfiles.get("f_zshrc").expect("f_zshrc not found");
    assert_eq!(zshrc.dtype, DotfileType::Link);

    let aliases = config
        .dotfiles
        .get("f_aliases")
        .expect("f_aliases not found");
    assert_eq!(aliases.dtype, DotfileType::Link);
}

#[test]
fn parse_missing_type_fails() {
    let content = "dotfiles:\n  f_bad:\n    dst: /tmp/x\n    src: y\n";
    let result = adot::parser::parse(content);
    let err = result.unwrap_err();
    assert!(err.contains("missing 'type'"), "got: {err}");
}

#[test]
fn parse_profile_include() {
    let path = PathBuf::from("tests/fixtures/config_link_default.yaml");
    let config = Config::load(Some(&path)).unwrap();

    let base = config.profiles.get("base").expect("base not found");
    assert_eq!(base.dotfiles, vec!["f_zshrc"]);
    assert!(base.include.is_empty());

    let host = config
        .profiles
        .get("test-host")
        .expect("test-host not found");
    assert_eq!(host.dotfiles, vec!["f_aliases"]);
    assert_eq!(host.include, vec!["base"]);
}

//////////////////////////////////////////////////////////////////////
// PARSER EDGE CASES

#[test]
fn parse_empty_yaml() {
    let result = adot::parser::parse("");
    let err = result.unwrap_err();
    assert!(err.contains("empty yaml document"), "got: {err}");
}

#[test]
fn parse_config_section_with_dotpath() {
    let content = "config:\n  dotpath: custom_dotfiles/\ndotfiles: {}\nprofiles: {}\n";
    let config = adot::parser::parse(content).unwrap();
    assert_eq!(config.dotpath, PathBuf::from("custom_dotfiles/"));
}

#[test]
fn parse_config_section_without_dotpath() {
    let content = "config:\n  backup: false\ndotfiles: {}\nprofiles: {}\n";
    let config = adot::parser::parse(content).unwrap();
    assert_eq!(config.dotpath, PathBuf::from("dotfiles"));
}

#[test]
fn parse_no_dotfiles_section() {
    let content = "profiles: {}\n";
    let config = adot::parser::parse(content).unwrap();
    assert!(config.dotfiles.is_empty());
}

#[test]
fn parse_no_profiles_section() {
    let content = "dotfiles: {}\n";
    let config = adot::parser::parse(content).unwrap();
    assert!(config.profiles.is_empty());
}

#[test]
fn parse_unknown_type_fails() {
    let content = "dotfiles:\n  f_bad:\n    dst: /tmp/x\n    src: y\n    type: banana\n";
    let err = adot::parser::parse(content).unwrap_err();
    assert!(err.contains("unknown type"), "got: {err}");
}

#[test]
fn parse_global_variables() {
    let content = r#"
dotfiles: {}
profiles: {}
variables:
  editor: vim
  colors:
    fg: white
    bg: black
"#;
    let config = adot::parser::parse(content).unwrap();
    assert_eq!(
        config.variables.get("editor"),
        Some(&Variable::Value("vim".to_string()))
    );

    match config.variables.get("colors") {
        Some(Variable::Nested(map)) => {
            assert_eq!(map.get("fg"), Some(&Variable::Value("white".to_string())));
            assert_eq!(map.get("bg"), Some(&Variable::Value("black".to_string())));
        }
        other => panic!("expected nested colors, got: {other:?}"),
    }
}

#[test]
fn parse_global_dynvariables() {
    let content = r#"
dotfiles: {}
profiles: {}
dynvariables:
  hostname: "hostname -s"
  os: "uname -s"
"#;
    let config = adot::parser::parse(content).unwrap();
    assert_eq!(
        config.dynvariables.get("hostname"),
        Some(&"hostname -s".to_string())
    );
    assert_eq!(config.dynvariables.get("os"), Some(&"uname -s".to_string()));
}

#[test]
fn parse_integer_variable() {
    let content = r#"
dotfiles: {}
profiles:
  test:
    dotfiles: []
    variables:
      thermal_zone: 3
      ratio: 1.5
      enabled: true
"#;
    let config = adot::parser::parse(content).unwrap();
    let profile = config.profiles.get("test").expect("test profile not found");
    assert_eq!(
        profile.variables.get("thermal_zone"),
        Some(&Variable::Value("3".to_string()))
    );
    assert_eq!(
        profile.variables.get("ratio"),
        Some(&Variable::Value("1.5".to_string()))
    );
    assert_eq!(
        profile.variables.get("enabled"),
        Some(&Variable::Value("true".to_string()))
    );
}

#[test]
fn parse_profile_dynvariables() {
    let content = r#"
dotfiles: {}
profiles:
  test:
    dotfiles: []
    dynvariables:
      user: "whoami"
"#;
    let config = adot::parser::parse(content).unwrap();
    let profile = config.profiles.get("test").expect("test profile not found");
    assert_eq!(
        profile.dynvariables.get("user"),
        Some(&"whoami".to_string())
    );
}

#[test]
fn parse_null_variable_becomes_empty_string() {
    let content = r#"
dotfiles: {}
profiles:
  test:
    dotfiles: []
    variables:
      empty_val:
"#;
    let config = adot::parser::parse(content).unwrap();
    let profile = config.profiles.get("test").expect("test profile not found");
    assert_eq!(
        profile.variables.get("empty_val"),
        Some(&Variable::Value("".to_string()))
    );
}

#[test]
fn parse_missing_dst_fails() {
    let content = "dotfiles:\n  f_bad:\n    src: y\n    type: link\n";
    let err = adot::parser::parse(content).unwrap_err();
    assert!(err.contains("missing 'dst'"), "got: {err}");
}

#[test]
fn parse_missing_src_fails() {
    let content = "dotfiles:\n  f_bad:\n    dst: /tmp/x\n    type: link\n";
    let err = adot::parser::parse(content).unwrap_err();
    assert!(err.contains("missing 'src'"), "got: {err}");
}

#[test]
fn parse_tilde_expansion_in_dst() {
    let content = "dotfiles:\n  f_test:\n    dst: ~/test_file\n    src: test\n    type: link\n";
    let config = adot::parser::parse(content).unwrap();
    let dotfile = config.dotfiles.get("f_test").unwrap();
    let home = std::env::var("HOME").unwrap();
    assert_eq!(dotfile.dst, PathBuf::from(format!("{home}/test_file")));
    // src shouldn't have ~ so stays as-is
    assert_eq!(dotfile.src, PathBuf::from("test"));
}

#[test]
fn parse_tilde_expansion_in_src() {
    let content = "dotfiles:\n  f_test:\n    dst: /tmp/out\n    src: ~/my_src\n    type: link\n";
    let config = adot::parser::parse(content).unwrap();
    let dotfile = config.dotfiles.get("f_test").unwrap();
    let home = std::env::var("HOME").unwrap();
    assert_eq!(dotfile.src, PathBuf::from(format!("{home}/my_src")));
}

#[test]
fn parse_tilde_no_home_stays_literal() {
    unsafe {
        let home = std::env::var("HOME").unwrap();
        std::env::remove_var("HOME");
        let content = "dotfiles:\n  f_test:\n    dst: ~/test\n    src: s\n    type: link\n";
        let config = adot::parser::parse(content).unwrap();
        let dotfile = config.dotfiles.get("f_test").unwrap();
        assert_eq!(dotfile.dst, PathBuf::from("~/test"));
        std::env::set_var("HOME", home);
    }
}

#[test]
fn parse_no_tilde_unchanged() {
    let content = "dotfiles:\n  f_test:\n    dst: /tmp/out\n    src: plain\n    type: link\n";
    let config = adot::parser::parse(content).unwrap();
    let dotfile = config.dotfiles.get("f_test").unwrap();
    assert_eq!(dotfile.dst, PathBuf::from("/tmp/out"));
    assert_eq!(dotfile.src, PathBuf::from("plain"));
}
