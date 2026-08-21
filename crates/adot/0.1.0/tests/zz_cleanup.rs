use std::path::Path;

#[test]
fn cleanup_tmp() {
    let dir = Path::new("/tmp/adot_tests");
    if dir.exists() {
        std::fs::remove_dir_all(dir).expect("failed to clean up /tmp/adot_tests");
    }
}
