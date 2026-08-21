use std::path::PathBuf;

use adot::config::Config;
use adot::installer::Installer;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

//////////////////////////////////////////////////////////////////////
// TEST TEMPLATE INSTALL

#[test]
fn install_template_file() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_template/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (config, _) = Config::load(Some(&config_path)).unwrap();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let dst = dst_dir.join("config");
    assert!(dst.exists(), "template output should exist");

    let meta = dst.symlink_metadata().unwrap();
    assert!(
        !meta.file_type().is_symlink(),
        "template output should not be a symlink"
    );

    let content = std::fs::read_to_string(&dst).unwrap();
    let expected =
        "[user]\n    name = Test User\n    email = user@test.com\n[core]\n    editor = nvim\n";
    assert_eq!(content, expected);
}

#[test]
fn install_template_dir() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_template_dir/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_template_dir").unwrap().dst = dst_dir.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    // config.ini should have vars replaced
    let ini = dst_dir.join("config.ini");
    assert!(ini.exists());
    let content = std::fs::read_to_string(&ini).unwrap();
    let expected = "[colors]\nbackground = #000000\nforeground = #AABBCC\n";
    assert_eq!(content, expected);

    // conditional.conf — profile doesn't match "special-host"
    let conf = dst_dir.join("conditional.conf");
    assert!(conf.exists());
    let content = std::fs::read_to_string(&conf).unwrap();
    let expected = "base_setting = true\n";
    assert_eq!(content, expected);
}

#[test]
fn install_template_dir_with_matching_profile() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_template_dir_match/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_template_dir").unwrap().dst = dst_dir.clone();

    // add the special-host profile with same vars
    let test_profile = config.profiles.get("test-host").unwrap().clone();
    config
        .profiles
        .insert("special-host".to_string(), test_profile);

    let installer = Installer::new(config, "special-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let conf = dst_dir.join("conditional.conf");
    let content = std::fs::read_to_string(&conf).unwrap();
    let expected = "base_setting = true\nspecial_setting = enabled\n";
    assert_eq!(content, expected);
}

#[test]
fn install_template_overwrites_existing() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_template_overwrite/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);
    std::fs::create_dir_all(&dst_dir).unwrap();
    std::fs::write(dst_dir.join("config"), "old content").unwrap();

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_template").unwrap().dst = dst_dir.join("config");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let content = std::fs::read_to_string(dst_dir.join("config")).unwrap();
    assert!(content.contains("user@test.com"));
}

#[test]
fn install_template_idempotent() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_template_idempotent/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_template").unwrap().dst = dst_dir.join("config");

    let installer = Installer::new(
        config.clone(),
        "test-host".to_string(),
        fixtures_dir(),
        true,
    );
    installer.install().unwrap();

    // run again
    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let content = std::fs::read_to_string(dst_dir.join("config")).unwrap();
    assert!(content.contains("user@test.com"));
}

#[test]
fn install_template_nested_dir() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_template_nested/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_template_dir").unwrap().dst = dst_dir.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let nested = dst_dir.join("subdir/nested.conf");
    assert!(nested.exists(), "nested template file should exist");
    let content = std::fs::read_to_string(&nested).unwrap();
    assert_eq!(content, "nested_editor = nvim\n");
}

//////////////////////////////////////////////////////////////////////
// TEST TEMPLATE INSTALL ERROR PATHS

#[test]
fn install_template_unreadable_src_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_template_unreadable");
    let src_dir = base.join("dotfiles");
    if src_dir.exists() {
        let _ = std::fs::set_permissions(
            src_dir.join("locked_file"),
            std::fs::Permissions::from_mode(0o644),
        );
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&src_dir).unwrap();
    let locked_file = src_dir.join("locked_file");
    std::fs::write(&locked_file, "{{@@ editor @@}}").unwrap();
    std::fs::set_permissions(&locked_file, std::fs::Permissions::from_mode(0o000)).unwrap();

    let content = r#"
dotfiles:
  f_t:
    dst: /tmp/adot_tests/install_template_unreadable/dst/out
    src: locked_file
    type: template
profiles:
  test:
    dotfiles:
      - f_t
    variables:
      editor: vim
"#;
    let config = adot::parser::parse(content).unwrap();
    let installer = Installer::new(config, "test".to_string(), base.clone(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to read"), "got: {err}");

    std::fs::set_permissions(&locked_file, std::fs::Permissions::from_mode(0o644)).unwrap();
}

#[test]
fn install_template_write_readonly_dst_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_template_write_readonly");
    let locked_dir = base.join("locked");
    if locked_dir.exists() {
        let _ = std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked_dir).unwrap();
    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_template").unwrap().dst = locked_dir.join("output");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to write"), "got: {err}");

    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn install_template_skips_unchanged_file() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_template_skip_unchanged/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);
    std::fs::create_dir_all(&dst_dir).unwrap();

    // pre-write the exact rendered content
    let expected = "[user]\n    name = Test User\n    email = user@test.com\n[core]\n    editor = nvim\n";
    std::fs::write(dst_dir.join("config"), expected).unwrap();

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_template").unwrap().dst = dst_dir.join("config");

    let mtime_before = std::fs::metadata(dst_dir.join("config")).unwrap().modified().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let mtime_after = std::fs::metadata(dst_dir.join("config")).unwrap().modified().unwrap();
    assert_eq!(mtime_before, mtime_after, "template was rewritten despite matching content");
}

#[test]
fn install_template_create_parent_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_template_create_parent_fails");
    let locked = base.join("locked");
    if locked.exists() {
        let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_template").unwrap().dst = locked.join("newdir/output");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to create dir"), "got: {err}");

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn install_template_missing_src_fails() {
    let content = r#"
dotfiles:
  f_ghost:
    dst: /tmp/adot_tests/install_template_ghost/dst/out
    src: does_not_exist
    type: template
profiles:
  test:
    dotfiles:
      - f_ghost
    variables:
      editor: vim
"#;
    let config = adot::parser::parse(content).unwrap();
    let installer = Installer::new(config, "test".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("src does not exist"), "got: {err}");
}

#[test]
fn install_template_dir_unreadable_src_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_template_dir_unreadable");
    let src_dir = base.join("dotfiles");
    let locked = src_dir.join("locked_tpl_dir");
    if locked.exists() {
        let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked).unwrap();
    std::fs::write(locked.join("file.conf"), "{{@@ editor @@}}").unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

    let content = r#"
dotfiles:
  d_t:
    dst: /tmp/adot_tests/install_template_dir_unreadable/dst/out
    src: locked_tpl_dir
    type: template
profiles:
  test:
    dotfiles:
      - d_t
    variables:
      editor: vim
"#;
    let config = adot::parser::parse(content).unwrap();
    let installer = Installer::new(config, "test".to_string(), base.clone(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to read dir"), "got: {err}");

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn install_template_nested_bad_variable_fails() {
    let base = PathBuf::from("/tmp/adot_tests/install_template_nested_badvar");
    let src_dir = base.join("dotfiles/tpl_dir/subdir");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("bad.conf"), "{{@@ missing_var @@}}").unwrap();

    let content = r#"
dotfiles:
  d_t:
    dst: /tmp/adot_tests/install_template_nested_badvar/dst/out
    src: tpl_dir
    type: template
profiles:
  test:
    dotfiles:
      - d_t
    variables:
      editor: vim
"#;
    let config = adot::parser::parse(content).unwrap();
    let installer = Installer::new(config, "test".to_string(), base.clone(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("undefined variable"), "got: {err}");
}

#[test]
fn install_template_bad_variable_fails() {
    let content = r#"
dotfiles:
  f_t:
    dst: /tmp/adot_tests/install_template_badvar/dst/out
    src: template_file
    type: template
profiles:
  test:
    dotfiles:
      - f_t
    variables:
      wrong_var: value
"#;
    let config = adot::parser::parse(content).unwrap();
    let installer = Installer::new(config, "test".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("undefined variable"), "got: {err}");
}

#[test]
fn install_template_dir_readonly_dst_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_template_dir_readonly");
    let locked_dir = base.join("locked");
    if locked_dir.exists() {
        let _ = std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked_dir).unwrap();
    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_template.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_template_dir").unwrap().dst = locked_dir.join("subdir");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(
        err.contains("failed to create dir") || err.contains("failed to write"),
        "got: {err}"
    );

    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}
