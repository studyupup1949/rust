use super::*;

#[cfg(unix)]
#[test]
fn display_is_collision_free_for_invalid_bytes_and_percent_literals() {
    use std::os::unix::ffi::OsStrExt;
    let invalid = OsStr::from_bytes(b"x\x80");
    assert_eq!(display_component(invalid), "%78%80");
    assert_eq!(display_component(OsStr::new("%78%80")), "%2578%2580");
}

#[test]
fn display_escapes_ambiguous_valid_characters() {
    assert_eq!(display_component(OsStr::new("a%b\\c\n")), "a%25b%5Cc%0A");
}

#[cfg(windows)]
#[test]
fn display_preserves_unpaired_utf16_without_collision() {
    use std::os::windows::ffi::OsStringExt;

    let invalid = OsString::from_wide(&[0xD800]);
    assert_eq!(display_component(&invalid), "%uD800");
    assert_eq!(display_component(OsStr::new("%uD800")), "%25uD800");
}

#[cfg(unix)]
#[test]
fn final_file_replacement_is_detected_before_content_read() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("item"), b"inside").unwrap();
    std::fs::write(outside.path().join("canary"), b"outside-secret").unwrap();
    let paths = collect(root.path()).unwrap();
    let Node::File(file) = &paths.get(Path::new("item")).unwrap().node else {
        panic!()
    };

    let result = read_file_with(file, || {
        std::fs::remove_file(root.path().join("item")).unwrap();
        symlink(outside.path().join("canary"), root.path().join("item")).unwrap();
        Ok(())
    });
    let issue = result.expect_err("replacement must not be read");
    assert_eq!(issue.code, "AAAI-PATH-RACE");
}

#[cfg(unix)]
#[test]
fn directory_to_fifo_replacement_is_rejected_without_blocking() {
    use std::process::Command;

    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("child")).unwrap();
    let result = collect_with_hook(root.path(), &|relative, phase| {
        if relative == Path::new("child") && phase == OpenPhase::Directory {
            std::fs::remove_dir(root.path().join("child")).unwrap();
            assert!(
                Command::new("mkfifo")
                    .arg(root.path().join("child"))
                    .status()
                    .unwrap()
                    .success()
            );
        }
        Ok(())
    })
    .unwrap();
    let Node::Issue(issue) = &result.get(Path::new("child")).unwrap().node else {
        panic!()
    };
    assert_eq!(issue.code, "AAAI-PATH-RACE");
}

#[cfg(unix)]
#[test]
fn directory_to_outside_link_replacement_is_not_traversed() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("child")).unwrap();
    std::fs::write(
        outside.path().join("outside-secret-name"),
        b"outside-secret-content",
    )
    .unwrap();
    let result = collect_with_hook(root.path(), &|relative, phase| {
        if relative == Path::new("child") && phase == OpenPhase::Directory {
            std::fs::remove_dir(root.path().join("child")).unwrap();
            symlink(outside.path(), root.path().join("child")).unwrap();
        }
        Ok(())
    })
    .unwrap();
    let Node::Issue(issue) = &result.get(Path::new("child")).unwrap().node else {
        panic!()
    };
    assert_eq!(issue.code, "AAAI-PATH-RACE");
    assert_eq!(
        result.len(),
        1,
        "the replacement target must not be enumerated"
    );
}

#[cfg(unix)]
#[test]
fn final_file_to_fifo_replacement_is_rejected_without_blocking() {
    use std::process::Command;

    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("item"), b"inside").unwrap();
    let paths = collect(root.path()).unwrap();
    let Node::File(file) = &paths.get(Path::new("item")).unwrap().node else {
        panic!()
    };
    let result = read_file_with(file, || {
        std::fs::remove_file(root.path().join("item")).unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(root.path().join("item"))
                .status()
                .unwrap()
                .success()
        );
        Ok(())
    });
    let issue = result.expect_err("FIFO replacement must not be read");
    assert_eq!(issue.code, "AAAI-PATH-RACE");
}

#[test]
fn regular_open_permission_failure_is_path_local_unreadable() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("blocked"), b"inside").unwrap();
    std::fs::write(root.path().join("safe"), b"safe").unwrap();
    let paths = collect(root.path()).unwrap();
    let Node::File(file) = &paths.get(Path::new("blocked")).unwrap().node else { panic!() };

    let result = read_file_with(file, || {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "synthetic path-local permission denial",
        ))
    });
    let issue = result.expect_err("permission denial must be unreadable");
    assert_eq!(issue.code, "AAAI-PATH-READ");
    assert!(issue.unreadable);
    assert!(matches!(paths.get(Path::new("safe")).unwrap().node, Node::File(_)));
}

#[test]
fn directory_open_permission_failure_preserves_unrelated_results() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("blocked-dir")).unwrap();
    std::fs::write(root.path().join("safe"), b"safe").unwrap();
    let paths = collect_with_hook(root.path(), &|relative, phase| {
        if relative == Path::new("blocked-dir") && phase == OpenPhase::Directory {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "synthetic path-local permission denial",
            ));
        }
        Ok(())
    }).unwrap();

    let Node::Issue(issue) = &paths.get(Path::new("blocked-dir")).unwrap().node else { panic!() };
    assert_eq!(issue.code, "AAAI-PATH-READ");
    assert!(issue.unreadable);
    assert!(matches!(paths.get(Path::new("safe")).unwrap().node, Node::File(_)));
}

#[test]
fn descendant_enumeration_failure_is_path_local_unreadable() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("blocked-dir")).unwrap();
    std::fs::write(root.path().join("blocked-dir").join("hidden"), b"hidden").unwrap();
    std::fs::write(root.path().join("safe"), b"safe").unwrap();
    let paths = collect_with_hook(root.path(), &|relative, phase| {
        if relative == Path::new("blocked-dir") && phase == OpenPhase::Enumerate {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "synthetic path-local enumeration denial",
            ));
        }
        Ok(())
    }).unwrap();

    let Node::Issue(issue) = &paths.get(Path::new("blocked-dir")).unwrap().node else { panic!() };
    assert_eq!(issue.code, "AAAI-PATH-READ");
    assert!(issue.unreadable);
    assert!(!paths.contains_key(Path::new("blocked-dir/hidden")));
    assert!(matches!(paths.get(Path::new("safe")).unwrap().node, Node::File(_)));
}

#[test]
fn removed_file_and_directory_are_races_not_unreadable_io() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("removed-file"), b"inside").unwrap();
    std::fs::create_dir(root.path().join("removed-dir")).unwrap();
    std::fs::write(root.path().join("safe"), b"safe").unwrap();

    let paths = collect(root.path()).unwrap();
    let Node::File(file) = &paths.get(Path::new("removed-file")).unwrap().node else { panic!() };
    let result = read_file_with(file, || {
        std::fs::remove_file(root.path().join("removed-file"))?;
        Ok(())
    });
    let issue = result.expect_err("removed file must be a race");
    assert_eq!(issue.code, "AAAI-PATH-RACE");
    assert!(!issue.unreadable);

    let paths = collect_with_hook(root.path(), &|relative, phase| {
        if relative == Path::new("removed-dir") && phase == OpenPhase::Directory {
            std::fs::remove_dir(root.path().join("removed-dir"))?;
        }
        Ok(())
    }).unwrap();
    let Node::Issue(issue) = &paths.get(Path::new("removed-dir")).unwrap().node else { panic!() };
    assert_eq!(issue.code, "AAAI-PATH-RACE");
    assert!(!issue.unreadable);
    assert!(matches!(paths.get(Path::new("safe")).unwrap().node, Node::File(_)));
}

#[cfg(windows)]
#[test]
fn windows_final_file_to_outside_link_replacement_is_rejected() {
    use std::os::windows::fs::symlink_file;

    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("item"), b"inside").unwrap();
    let canary = outside.path().join("canary");
    std::fs::write(&canary, b"outside-secret-content").unwrap();
    let before_canary = std::fs::read(&canary).unwrap();
    let paths = collect(root.path()).unwrap();
    let Node::File(file) = &paths.get(Path::new("item")).unwrap().node else { panic!() };
    let result = read_file_with(file, || {
        std::fs::remove_file(root.path().join("item")).unwrap();
        symlink_file(&canary, root.path().join("item"))
            .expect("hosted Windows file-symlink race fixture");
        Ok(())
    });
    let issue = result.expect_err("outside-link replacement must not be read");
    assert_eq!(issue.code, "AAAI-PATH-RACE");
    assert_eq!(
        issue.detail,
        "The file changed to a reparse point before it was read."
    );
    assert!(!issue_text(&issue).contains("outside-secret-content"));
    assert_eq!(std::fs::read(&canary).unwrap(), before_canary);
}

#[cfg(windows)]
#[test]
fn windows_directory_to_outside_link_replacement_is_not_traversed() {
    use std::os::windows::fs::symlink_dir;

    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("child")).unwrap();
    std::fs::write(outside.path().join("outside-secret-name"), b"outside-secret-content").unwrap();
    let result = collect_with_hook(root.path(), &|relative, phase| {
        if relative == Path::new("child") && phase == OpenPhase::Directory {
            std::fs::remove_dir(root.path().join("child")).unwrap();
            symlink_dir(outside.path(), root.path().join("child"))
                .expect("hosted Windows directory-symlink race fixture");
        }
        Ok(())
    }).unwrap();
    let Node::Issue(issue) = &result.get(Path::new("child")).unwrap().node else { panic!() };
    assert_eq!(issue.code, "AAAI-PATH-RACE");
    assert_eq!(result.len(), 1, "the replacement target must not be enumerated");
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn real_mounted_child_is_rejected_through_production_xdev_check() {
    let mut candidates: Vec<(PathBuf, OsString)> = Vec::new();
    #[cfg(target_os = "linux")]
    {
        candidates.push((PathBuf::from("/"), OsString::from("proc")));
        candidates.push((PathBuf::from("/dev"), OsString::from("shm")));
    }
    #[cfg(target_os = "macos")]
    {
        candidates.push((PathBuf::from("/"), OsString::from("dev")));
        candidates.push((PathBuf::from("/System/Volumes"), OsString::from("Data")));
        if let Ok(entries) = std::fs::read_dir("/Volumes") {
            candidates.extend(
                entries
                    .filter_map(Result::ok)
                    .map(|entry| (PathBuf::from("/Volumes"), entry.file_name())),
            );
        }
    }

    let mut observed = false;
    for (parent_path, child_name) in candidates {
        let Ok(parent) = open_selected_root(&parent_path) else {
            continue;
        };
        let Ok(parent_metadata) = parent.dir_metadata() else {
            continue;
        };
        let root_device = identity(&parent_metadata).device;
        let opened = open_directory_nofollow(&parent, &child_name);
        if let Err(path_issue) =
            classify_directory(&parent, &child_name, root_device, opened)
            && path_issue.code == "AAAI-PATH-XDEV"
        {
            observed = true;
            break;
        }
    }
    assert!(
        observed,
        "a real accessible differing-device mounted child is required"
    );
}
