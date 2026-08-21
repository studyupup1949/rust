use super::*;

#[test]
fn lock_acquire_and_release() {
    let tmp = tempfile::tempdir().unwrap();
    let def = tmp.path().join("audit.yaml");
    std::fs::write(&def, "").unwrap();

    {
        let _guard = acquire(&def).unwrap();
        let lock = def.with_extension("lock");
        assert!(lock.exists(), "lock file should exist while guard is alive");
    }

    let lock = def.with_extension("lock");
    assert!(!lock.exists(), "lock file should be removed on drop");
}

#[test]
fn double_lock_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let def = tmp.path().join("audit.yaml");
    std::fs::write(&def, "").unwrap();

    let _guard1 = acquire(&def).unwrap();
    let result2 = acquire(&def);
    assert!(result2.is_err(), "second acquire should fail while lock is held");
}
