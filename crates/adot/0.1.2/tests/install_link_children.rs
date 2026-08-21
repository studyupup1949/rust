use std::path::PathBuf;

use adot::config::Config;
use adot::installer::Installer;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

//////////////////////////////////////////////////////////////////////
// TEST LINK_CHILDREN

#[test]
fn install_link_children_creates_symlinks_per_child() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link_children/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_link_children.yaml");
    let config = Config::load(Some(&config_path)).unwrap();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    // dst should be a real directory, not a symlink
    assert!(dst_dir.exists());
    let meta = dst_dir.symlink_metadata().unwrap();
    assert!(
        !meta.file_type().is_symlink(),
        "dst dir itself should not be a symlink"
    );
    assert!(meta.is_dir());

    // each child should be a symlink
    let child_file = dst_dir.join("file_in_dir");
    assert!(child_file.exists(), "file_in_dir should exist");
    let meta = child_file.symlink_metadata().unwrap();
    assert!(
        meta.file_type().is_symlink(),
        "child file should be a symlink"
    );
    let target = std::fs::read_link(&child_file).unwrap();
    assert_eq!(target, fixtures_dir().join("dotfiles/dir/file_in_dir"));

    let child_subdir = dst_dir.join("subdir");
    assert!(child_subdir.exists(), "subdir should exist");
    let meta = child_subdir.symlink_metadata().unwrap();
    assert!(
        meta.file_type().is_symlink(),
        "child subdir should be a symlink"
    );
    let target = std::fs::read_link(&child_subdir).unwrap();
    assert_eq!(target, fixtures_dir().join("dotfiles/dir/subdir"));

    // nested file accessible through symlinked subdir
    let nested = child_subdir.join("nested_file");
    assert!(
        nested.exists(),
        "nested_file should be accessible through symlinked subdir"
    );
    assert_eq!(std::fs::read_to_string(&nested).unwrap(), "nested\n");
}

#[test]
fn install_link_children_overwrites_existing_children() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link_children_overwrite/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);
    std::fs::create_dir_all(&dst_dir).unwrap();

    // create stale files where children should go
    std::fs::write(dst_dir.join("file_in_dir"), "stale").unwrap();
    std::fs::create_dir_all(dst_dir.join("subdir")).unwrap();
    std::fs::write(dst_dir.join("subdir/old"), "old").unwrap();

    let config_path = fixtures_dir().join("config_install_link_children.yaml");
    let mut config = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_dir").unwrap().dst = dst_dir.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    // children should now be symlinks
    let child_file = dst_dir.join("file_in_dir");
    let meta = child_file.symlink_metadata().unwrap();
    assert!(
        meta.file_type().is_symlink(),
        "should be replaced with symlink"
    );
    assert_eq!(
        std::fs::read_to_string(&child_file).unwrap(),
        "inside dir\n"
    );

    let child_subdir = dst_dir.join("subdir");
    let meta = child_subdir.symlink_metadata().unwrap();
    assert!(
        meta.file_type().is_symlink(),
        "subdir should be replaced with symlink"
    );
}

#[test]
fn install_link_children_idempotent() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_link_children_idempotent/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_link_children.yaml");
    let mut config = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_dir").unwrap().dst = dst_dir.clone();

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

    let child_file = dst_dir.join("file_in_dir");
    let meta = child_file.symlink_metadata().unwrap();
    assert!(meta.file_type().is_symlink());
    assert_eq!(
        std::fs::read_to_string(&child_file).unwrap(),
        "inside dir\n"
    );
}

#[test]
fn install_link_children_fails_on_file_src() {
    let config_path = fixtures_dir().join("config_install_link_children.yaml");
    let mut config = Config::load(Some(&config_path)).unwrap();
    // point src to a file instead of a dir
    config.dotfiles.get_mut("d_dir").unwrap().src = PathBuf::from("file");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("requires a directory"), "got: {err}");
}

#[test]
fn install_link_children_missing_src_fails() {
    let config_path = fixtures_dir().join("config_install_link_children.yaml");
    let mut config = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_dir").unwrap().src = PathBuf::from("does_not_exist");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("src does not exist"), "got: {err}");
}

#[test]
fn install_link_children_readonly_dst_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_link_children_readonly");
    let locked_dir = base.join("locked");
    if locked_dir.exists() {
        let _ = std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked_dir).unwrap();
    // put an existing file so the symlink creation fails
    std::fs::write(locked_dir.join("file_in_dir"), "stale").unwrap();
    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_link_children.yaml");
    let mut config = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_dir").unwrap().dst = locked_dir.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(
        err.contains("failed to remove existing") || err.contains("failed to symlink"),
        "got: {err}"
    );

    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn install_link_children_create_dst_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_link_children_create_fails");
    let locked_parent = base.join("locked");
    if locked_parent.exists() {
        let _ = std::fs::set_permissions(&locked_parent, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked_parent).unwrap();
    std::fs::set_permissions(&locked_parent, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_link_children.yaml");
    let mut config = Config::load(Some(&config_path)).unwrap();
    // dst is inside a readonly parent, so create_dir_all will fail
    config.dotfiles.get_mut("d_dir").unwrap().dst = locked_parent.join("new_subdir");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to create dir"), "got: {err}");

    std::fs::set_permissions(&locked_parent, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn install_link_children_unreadable_src_dir_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_link_children_unreadable_src");
    let src_dir = base.join("dotfiles/locked_dir");
    if src_dir.exists() {
        let _ = std::fs::set_permissions(&src_dir, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("child"), "data").unwrap();
    std::fs::set_permissions(&src_dir, std::fs::Permissions::from_mode(0o000)).unwrap();

    let config_path = fixtures_dir().join("config_install_link_children.yaml");
    let mut config = Config::load(Some(&config_path)).unwrap();
    config.dotpath = std::path::PathBuf::from(".");
    config.dotfiles.get_mut("d_dir").unwrap().src = PathBuf::from("locked_dir");
    config.dotfiles.get_mut("d_dir").unwrap().dst = base.join("dst");

    let installer = Installer::new(config, "test-host".to_string(), base.join("dotfiles"), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to read dir"), "got: {err}");

    std::fs::set_permissions(&src_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}
