use std::path::PathBuf;

use adot::config::{Config, resolve_config_path};

#[test]
fn resolve_overwrite_exists() {
    let path = PathBuf::from("tests/fixtures/config_copy.yaml");
    let result = resolve_config_path(Some(&path)).unwrap();
    assert_eq!(result, path);
}

#[test]
fn resolve_overwrite_missing() {
    let path = PathBuf::from("tests/fixtures/does_not_exist.yaml");
    let err = resolve_config_path(Some(&path)).unwrap_err();
    assert!(err.contains("config not found"), "got: {err}");
}

#[test]
fn resolve_no_overwrite_no_config() {
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/adot_tests/fake_xdg");
        std::env::set_var("HOME", "/tmp/adot_tests/fake_home");
    }
    let err = resolve_config_path(None).unwrap_err();
    assert!(err.contains("no config found"), "got: {err}");
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("HOME");
    }
}

#[test]
fn resolve_no_xdg_falls_through_to_home() {
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("HOME", "/tmp/adot_tests/fake_home_no_xdg");
    }
    let err = resolve_config_path(None).unwrap_err();
    assert!(err.contains("no config found"), "got: {err}");
    unsafe {
        std::env::remove_var("HOME");
    }
}

#[test]
fn resolve_xdg_config_home() {
    let dir = PathBuf::from("/tmp/adot_tests/xdg_resolve/adot");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("config.yaml"), "dotfiles: {}").unwrap();

    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/adot_tests/xdg_resolve");
    }
    let result = resolve_config_path(None).unwrap();
    assert_eq!(result, dir.join("config.yaml"));
    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    std::fs::remove_dir_all("/tmp/adot_tests/xdg_resolve").unwrap();
}

#[test]
fn load_nonexistent_file() {
    let path = PathBuf::from("tests/fixtures/nope.yaml");
    let err = Config::load(Some(&path)).unwrap_err();
    assert!(err.contains("config not found"), "got: {err}");
}

#[test]
fn load_invalid_yaml() {
    let dir = "/tmp/adot_tests/invalid_yaml";
    std::fs::create_dir_all(dir).unwrap();
    let path = PathBuf::from(format!("{dir}/config.yaml"));
    std::fs::write(&path, "{{{{ not valid yaml: [[[").unwrap();

    let err = Config::load(Some(&path)).unwrap_err();
    assert!(err.contains("yaml parse error"), "got: {err}");
}

#[test]
fn load_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;

    let dir = "/tmp/adot_tests/unreadable";
    std::fs::create_dir_all(dir).unwrap();
    let path = PathBuf::from(format!("{dir}/config.yaml"));
    std::fs::write(&path, "dotfiles: {}").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

    let err = Config::load(Some(&path)).unwrap_err();
    assert!(err.contains("failed to read"), "got: {err}");

    // restore permissions so cleanup can remove it
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
}

#[test]
fn validate_src_equals_dst() {
    let path = PathBuf::from("tests/fixtures/config_invalid_src_eq_dst.yaml");
    let err = Config::load(Some(&path)).unwrap_err();
    assert!(err.contains("src and dst are the same path"), "got: {err}");
}

#[test]
fn validate_dst_inside_dotpath() {
    let path = PathBuf::from("tests/fixtures/config_invalid_dst_in_dotpath.yaml");
    let err = Config::load(Some(&path)).unwrap_err();
    assert!(err.contains("inside dotpath"), "got: {err}");
}

#[test]
fn validate_unknown_dotfile_ref() {
    let path = PathBuf::from("tests/fixtures/config_invalid_unknown_dotfile_ref.yaml");
    let err = Config::load(Some(&path)).unwrap_err();
    assert!(err.contains("unknown dotfile"), "got: {err}");
}

#[test]
fn validate_unknown_include() {
    let path = PathBuf::from("tests/fixtures/config_invalid_unknown_include.yaml");
    let err = Config::load(Some(&path)).unwrap_err();
    assert!(err.contains("unknown profile"), "got: {err}");
}

#[test]
fn validate_empty_dst() {
    let path = PathBuf::from("tests/fixtures/config_invalid_empty_dst.yaml");
    let err = Config::load(Some(&path)).unwrap_err();
    assert!(err.contains("dst is empty"), "got: {err}");
}

#[test]
fn validate_empty_src() {
    let path = PathBuf::from("tests/fixtures/config_invalid_empty_src.yaml");
    let err = Config::load(Some(&path)).unwrap_err();
    assert!(err.contains("src is empty"), "got: {err}");
}
