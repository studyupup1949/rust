use std::path::PathBuf;

use adot::config::Config;
use adot::installer::Installer;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

//////////////////////////////////////////////////////////////////////
// TEST COPY

#[test]
fn install_copy_file_and_dir() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_copy/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_copy.yaml");
    let (config, _) = Config::load(Some(&config_path)).unwrap();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    // check file copy — should NOT be a symlink
    let file_dst = dst_dir.join("file");
    assert!(file_dst.exists(), "dst file should exist");
    let meta = file_dst.symlink_metadata().unwrap();
    assert!(
        !meta.file_type().is_symlink(),
        "copy should not be a symlink"
    );
    assert_eq!(std::fs::read_to_string(&file_dst).unwrap(), "hello\n");

    // check dir copy — should NOT be a symlink
    let dir_dst = dst_dir.join("dir");
    assert!(dir_dst.exists(), "dst dir should exist");
    let meta = dir_dst.symlink_metadata().unwrap();
    assert!(
        !meta.file_type().is_symlink(),
        "copy should not be a symlink"
    );
    assert!(dir_dst.is_dir(), "should be a real directory");

    let inner = dir_dst.join("file_in_dir");
    assert!(inner.exists(), "file_in_dir should exist in copied dir");
    assert_eq!(std::fs::read_to_string(&inner).unwrap(), "inside dir\n");

    // nested subdir should be copied recursively
    let nested = dir_dst.join("subdir/nested_file");
    assert!(nested.exists(), "nested file should exist in copied subdir");
    assert_eq!(std::fs::read_to_string(&nested).unwrap(), "nested\n");
}

#[test]
fn install_copy_is_independent_of_source() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_copy_independent/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_copy.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    let file_dst = dst_dir.join("file");
    config.dotfiles.get_mut("f_file").unwrap().dst = file_dst.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    // modify the copy — source should be unaffected
    std::fs::write(&file_dst, "modified").unwrap();
    assert_eq!(std::fs::read_to_string(&file_dst).unwrap(), "modified");

    let src = fixtures_dir().join("dotfiles/file");
    assert_eq!(std::fs::read_to_string(&src).unwrap(), "hello\n");
}

#[test]
fn install_copy_overwrites_existing() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_copy_overwrite/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);
    std::fs::create_dir_all(&dst_dir).unwrap();

    let file_dst = dst_dir.join("file");
    std::fs::write(&file_dst, "old content").unwrap();

    let config_path = fixtures_dir().join("config_install_copy.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_file").unwrap().dst = file_dst.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    let meta = file_dst.symlink_metadata().unwrap();
    assert!(
        !meta.file_type().is_symlink(),
        "should be a regular file, not symlink"
    );
    assert_eq!(std::fs::read_to_string(&file_dst).unwrap(), "hello\n");
}

#[test]
fn install_copy_idempotent() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_copy_idempotent/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);

    let config_path = fixtures_dir().join("config_install_copy.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    for df in config.dotfiles.values_mut() {
        let name = df.dst.file_name().unwrap().to_owned();
        df.dst = dst_dir.join(name);
    }

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

    let file_dst = dst_dir.join("file");
    assert!(file_dst.exists());
    assert_eq!(std::fs::read_to_string(&file_dst).unwrap(), "hello\n");
}

#[test]
fn install_copy_unreadable_src_dir_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_copy_unreadable");
    let src_dir = base.join("dotfiles");
    let unreadable = src_dir.join("locked_dir");
    // restore perms from previous run so we can clean up
    if unreadable.exists() {
        let _ = std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&unreadable).unwrap();
    std::fs::write(unreadable.join("secret"), "data").unwrap();
    // create the file src so it doesn't fail before reaching the dir copy
    std::fs::write(src_dir.join("file"), "hello").unwrap();
    // make the dir unreadable
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000)).unwrap();

    let dst_dir = base.join("dst");

    let config_path = fixtures_dir().join("config_install_copy.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_dir").unwrap().src = PathBuf::from("locked_dir");
    config.dotfiles.get_mut("d_dir").unwrap().dst = dst_dir.join("locked_dir");
    config.dotfiles.get_mut("f_file").unwrap().src = PathBuf::from("file");
    config.dotfiles.get_mut("f_file").unwrap().dst = dst_dir.join("file");

    let installer = Installer::new(config, "test-host".to_string(), base.clone(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to copy"), "got: {err}");

    // restore permissions so cleanup can remove it
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn install_copy_single_file_readonly_dst_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_copy_readonly_dst");
    let locked_dir = base.join("locked");
    if locked_dir.exists() {
        let _ = std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked_dir).unwrap();
    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_copy.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_file").unwrap().dst = locked_dir.join("file");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(
        err.contains("failed to copy") || err.contains("failed to create dir"),
        "got: {err}"
    );

    std::fs::set_permissions(&locked_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn install_copy_missing_src_fails() {
    let content = r#"
dotfiles:
  f_ghost:
    dst: /tmp/adot_tests/install_copy_ghost/dst/file
    src: does_not_exist
    type: copy
profiles:
  test-host:
    dotfiles:
      - f_ghost
"#;
    let config = adot::parser::parse(content).unwrap();
    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("src does not exist"), "got: {err}");
}

#[test]
fn install_copy_dir_does_not_destroy_existing_contents() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_copy_merge/dst/dir");
    let _ = std::fs::remove_dir_all("/tmp/adot_tests/install_copy_merge");
    std::fs::create_dir_all(&dst_dir).unwrap();

    // pre-existing file that is NOT in the source
    std::fs::write(dst_dir.join("existing_file"), "must survive").unwrap();
    // pre-existing subdir with content
    std::fs::create_dir_all(dst_dir.join("existing_subdir")).unwrap();
    std::fs::write(dst_dir.join("existing_subdir/data"), "also survives").unwrap();

    let config_path = fixtures_dir().join("config_install_copy.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("d_dir").unwrap().dst = dst_dir.clone();

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    // source files should be copied
    assert_eq!(
        std::fs::read_to_string(dst_dir.join("file_in_dir")).unwrap(),
        "inside dir\n"
    );
    assert_eq!(
        std::fs::read_to_string(dst_dir.join("subdir/nested_file")).unwrap(),
        "nested\n"
    );

    // pre-existing files must still be there
    assert!(
        dst_dir.join("existing_file").exists(),
        "existing_file was destroyed"
    );
    assert_eq!(
        std::fs::read_to_string(dst_dir.join("existing_file")).unwrap(),
        "must survive"
    );
    assert!(
        dst_dir.join("existing_subdir/data").exists(),
        "existing_subdir/data was destroyed"
    );
    assert_eq!(
        std::fs::read_to_string(dst_dir.join("existing_subdir/data")).unwrap(),
        "also survives"
    );
}

#[test]
fn install_copy_skips_unchanged_file() {
    let dst_dir = PathBuf::from("/tmp/adot_tests/install_copy_skip_unchanged/dst");
    let _ = std::fs::remove_dir_all(&dst_dir);
    std::fs::create_dir_all(&dst_dir).unwrap();

    // pre-write the exact content that copy would produce
    std::fs::write(dst_dir.join("file"), "hello\n").unwrap();

    let config_path = fixtures_dir().join("config_install_copy.yaml");
    let (mut config, _) = Config::load(Some(&config_path)).unwrap();
    config.dotfiles.get_mut("f_file").unwrap().dst = dst_dir.join("file");

    // record mtime before
    let mtime_before = std::fs::metadata(dst_dir.join("file")).unwrap().modified().unwrap();

    // small sleep to ensure mtime would differ if file was written
    std::thread::sleep(std::time::Duration::from_millis(50));

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    installer.install().unwrap();

    // mtime should be unchanged — file was not rewritten
    let mtime_after = std::fs::metadata(dst_dir.join("file")).unwrap().modified().unwrap();
    assert_eq!(mtime_before, mtime_after, "file was rewritten despite matching content");
}

#[test]
fn install_copy_single_file_create_parent_fails() {
    use std::os::unix::fs::PermissionsExt;

    let base = PathBuf::from("/tmp/adot_tests/install_copy_create_parent_fails");
    let locked = base.join("locked");
    if locked.exists() {
        let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();

    let config_path = fixtures_dir().join("config_install_copy.yaml");
    let mut config = Config::load(Some(&config_path)).unwrap().0;
    // dst parent needs to be created but grandparent is readonly
    config.dotfiles.get_mut("f_file").unwrap().dst = locked.join("newdir/file");

    let installer = Installer::new(config, "test-host".to_string(), fixtures_dir(), true);
    let err = installer.install().unwrap_err();
    assert!(err.contains("failed to create dir"), "got: {err}");

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
}
