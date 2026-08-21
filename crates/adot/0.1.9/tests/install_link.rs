use std::os::unix::fs as unix_fs;
use std::path::PathBuf;

use adot::config::Config;
use adot::installer::Installer;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

//////////////////////////////////////////////////////////////////////
// TEST LINK

#[test]
fn install_link_file_and_dir() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (config, _) = Config::load(Some(&config_path)).unwrap();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    // check file symlink
    let file_dst = dst_dir.join("file");
    assert!(file_dst.exists(), "dst file should exist");
    let meta = file_dst.symlink_metadata().unwrap();
    assert!(
        meta.file_type().is_symlink(),
        "file dst should be a symlink"
    );
    let target = std::fs::read_link(&file_dst).unwrap();
    assert_eq!(target, fixtures_dir().join("dotfiles/file"));
    assert_eq!(std::fs::read_to_string(&file_dst).unwrap(), "hello\n");

    // check dir symlink
    let dir_dst = dst_dir.join("dir");
    assert!(dir_dst.exists(), "dst dir should exist");
    let meta = dir_dst.symlink_metadata().unwrap();
    assert!(meta.file_type().is_symlink(), "dir dst should be a symlink");
    let target = std::fs::read_link(&dir_dst).unwrap();
    assert_eq!(target, fixtures_dir().join("dotfiles/dir"));
    let inner = dir_dst.join("file_in_dir");
    assert!(
        inner.exists(),
        "file_in_dir should be accessible through symlink"
    );
    assert_eq!(std::fs::read_to_string(&inner).unwrap(), "inside dir\n");
}

#[test]
fn install_link_overwrites_existing_file() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link_overwrite_file/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);
    std::fs::create_dir_all(&dst_dir).unwrap();

    let file_dst = dst_dir.join("file");
    std::fs::write(&file_dst, "old content").unwrap();

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    // repoint dst to our test dir
    config.dotfiles.get_mut("f_file").unwrap().dst = file_dst.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let meta = file_dst.symlink_metadata().unwrap();
    assert!(
        meta.file_type().is_symlink(),
        "file should be replaced with symlink"
    );
    assert_eq!(std::fs::read_to_string(&file_dst).unwrap(), "hello\n");
}

#[test]
fn install_link_does_not_replace_existing_dir() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link_no_replace_dir/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);
    std::fs::create_dir_all(&dst_dir).unwrap();

    let dir_dst = dst_dir.join("dir");
    std::fs::create_dir_all(&dir_dst).unwrap();
    std::fs::write(dir_dst.join("stale"), "stale").unwrap();

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_dir").unwrap().dst = dir_dst.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    // install will fail because it can't symlink over a directory
    let _ = installer.install();

    // directory must survive
    assert!(dir_dst.is_dir(), "directory must not be removed");
    assert_eq!(
        std::fs::read_to_string(dir_dst.join("stale")).unwrap(),
        "stale"
    );
}

#[test]
fn install_link_overwrites_existing_symlink() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link_overwrite_symlink/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);
    std::fs::create_dir_all(&dst_dir).unwrap();

    // create a stale symlink pointing somewhere else
    let file_dst = dst_dir.join("file");
    unix_fs::symlink("/tmp/nonexistent_target", &file_dst).unwrap();

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_file").unwrap().dst = file_dst.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let target = std::fs::read_link(&file_dst).unwrap();
    assert_eq!(target, fixtures_dir().join("dotfiles/file"));
    assert_eq!(std::fs::read_to_string(&file_dst).unwrap(), "hello\n");
}

#[test]
fn install_link_idempotent() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link_idempotent/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (config, _) = Config::load(Some(&config_path)).unwrap();

    let mut config2 = config.clone();
    for df in config2.dotfiles.values_mut() {
        let name = df.dst.file_name().unwrap().to_owned();
        df.dst = dst_dir.join(name);
    }

    let installer = Installer::new(
        config2.clone(),
        "test-host".to_string(),
        fixtures_dir(),
        true,
    );
    installer.install().unwrap();

    // run again — should not fail
    let installer = Installer::new(config2, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let file_dst = dst_dir.join("file");
    let meta = file_dst.symlink_metadata().unwrap();
    assert!(meta.file_type().is_symlink());
    assert_eq!(std::fs::read_to_string(&file_dst).unwrap(), "hello\n");
}

#[test]
fn install_link_creates_parent_dirs() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link_parents/deep/nested/dst");
    let _ = std::fs::remove_dir_all("/tmp/adot_tests/install_link_parents");

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_file").unwrap().dst = dst_dir.join("file");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let file_dst = dst_dir.join("file");
    assert!(file_dst.exists(), "file should exist in deeply nested dir");
    let meta = file_dst.symlink_metadata().unwrap();
    assert!(meta.file_type().is_symlink());
}

#[test]
fn install_link_missing_src_fails() {
    let config_path = fixtures_dir().join("config_install_link_missing_src.yaml");
    let (config, _) = Config::load(Some(&config_path)).unwrap();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("src does not exist"), "got: {err}");
}

#[test]
fn install_link_profile_not_found() {
    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (config, _) = Config::load(Some(&config_path)).unwrap();

    let installer = Installer::new(
        config,
        "nonexistent-profile".to_string(),
        fixtures_dir(),
        true,
    );
    let err = installer.install().unwrap_err();
    assert!(err.contains("not found"), "got: {err}");
}

#[test]
fn install_circular_include_fails() {
    let content = r#"
dotfiles:
  f_file:
    dst: /tmp/adot_tests/circular/dst/file
    src: file
    type: link
profiles:
  a:
    dotfiles:
      - f_file
    include:
      - b
  b:
    dotfiles: []
    include:
      - a
"#;
    let config = adot::parser::parse(content).unwrap();
    let installer = Installer::new(config, "a".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("circular"), "got: {err}");
}

#[test]
fn install_with_profile_include() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_include/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let content = r#"
dotfiles:
  f_file:
    dst: /tmp/adot_tests/install_include/dst/file
    src: file
    type: link
  d_dir:
    dst: /tmp/adot_tests/install_include/dst/dir
    src: dir
    type: link
profiles:
  base:
    dotfiles:
      - f_file
  child:
    dotfiles:
      - d_dir
    include:
      - base
"#;
    let config = adot::parser::parse(content).unwrap();
    let installer = Installer::new(config, "child".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    // both from base and child should be installed
    assert!(
        dst_dir.join("file").exists(),
        "file from base should be installed"
    );
    assert!(
        dst_dir.join("dir").exists(),
        "dir from child should be installed"
    );
}

//////////////////////////////////////////////////////////////////////
// TEST LINK ERROR PATHS

#[test]
fn install_link_readonly_parent_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_link_readonly_parent");
    let locked_dir = base.join("locked");
    if locked_dir.exists() {
        let _ = std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked_dir).unwrap();
    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_file").unwrap().dst = locked_dir.join("subdir/file");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to create dir"), "got: {err}");

    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn install_link_remove_readonly_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_link_remove_readonly");
    let locked_dir = base.join("locked");
    if locked_dir.exists() {
        let _ = std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked_dir).unwrap();
    // create a file that we can't remove (parent is read-only)
    std::fs::write(locked_dir.join("file"), "existing").unwrap();
    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_file").unwrap().dst = locked_dir.join("file");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to remove existing"), "got: {err}");

    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn install_link_symlink_readonly_dir_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_link_symlink_readonly");
    let locked_dir = base.join("locked");
    if locked_dir.exists() {
        let _ = std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked_dir).unwrap();
    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_file").unwrap().dst = locked_dir.join("file");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    // will fail on either create_dir or symlink
    assert!(
        err.contains("failed to create dir") || err.contains("failed to symlink"),
        "got: {err}"
    );

    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}

//////////////////////////////////////////////////////////////////////
// TEST SAFETY — NEVER REMOVE DIRECTORIES

#[test]
fn install_link_never_removes_existing_directory() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link_no_rm_dir/dst");
    let _ = std::fs::remove_dir_all("/tmp/adot_tests/install_link_no_rm_dir");
    std::fs::create_dir_all(&dst_dir).unwrap();

    // create a real directory at the dst path with content inside
    let target = dst_dir.join("file");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("must_survive"), "important data").unwrap();

    let config_path = fixtures_dir().join("config_install_link.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_file").unwrap().dst = target.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    // this should fail or skip — but NEVER remove the directory
    let _ = installer.install();

    // the directory and its contents MUST still exist
    assert!(target.exists(), "directory was removed — CATASTROPHIC");
    assert!(target.is_dir(), "directory was replaced — CATASTROPHIC");
    assert!(
        target.join("must_survive").exists(),
        "contents were destroyed — CATASTROPHIC"
    );
    assert_eq!(
        std::fs::read_to_string(target.join("must_survive")).unwrap(),
        "important data"
    );
}

#[test]
fn install_template_never_removes_existing_directory() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_template_no_rm_dir/dst");
    let _ = std::fs::remove_dir_all("/tmp/adot_tests/install_template_no_rm_dir");
    std::fs::create_dir_all(&dst_dir).unwrap();

    // dst is a real directory with important content
    let target = dst_dir.join("home");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("must_survive"), "important data").unwrap();

    let content = r##"
dotfiles:
  d_t:
    dst: /tmp/adot_tests/install_template_no_rm_dir/dst/home
    src: template_dir
    type: template
profiles:
  test:
    dotfiles:
      - d_t
    variables:
      editor: nvim
      colors:
        background: "#000"
        foreground: "#FFF"
"##;
    let config = adot::parser::parse(content).unwrap();
    let installer = Installer::new(config, "test".to_string(), fixtures_dir(), true);
    let _ = installer.install();

    // directory and pre-existing contents MUST survive
    assert!(target.exists(), "directory was removed — CATASTROPHIC");
    assert!(target.is_dir(), "directory was replaced — CATASTROPHIC");
    assert!(
        target.join("must_survive").exists(),
        "contents were destroyed — CATASTROPHIC"
    );
    assert_eq!(
        std::fs::read_to_string(target.join("must_survive")).unwrap(),
        "important data"
    );
}
