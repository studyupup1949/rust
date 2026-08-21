//! Integration tests requiring a running ADB server and connected device.
//!
//! Skipped by default. Run with:
//!
//! ```sh
//! cargo test -- --ignored
//! ```
//!
//! Set `ADB_SERIAL` to target a specific device (default: `emulator-5554`).

use adb_wire::{list_devices, AdbWire, DEFAULT_SERVER};

fn get_serial() -> String {
    std::env::var("ADB_SERIAL").unwrap_or_else(|_| "emulator-5554".to_string())
}

fn adb() -> AdbWire {
    AdbWire::new(&get_serial())
}

#[tokio::test]
#[ignore]
async fn list_devices_returns_entries() {
    let devices = list_devices(DEFAULT_SERVER).await.unwrap();
    assert!(!devices.is_empty(), "no devices connected");
}

#[tokio::test]
#[ignore]
async fn shell_echo() {
    let out = adb().shell("echo hello").await.unwrap();
    assert!(out.success());
    assert_eq!(out.stdout_str(), "hello");
}

#[tokio::test]
#[ignore]
async fn shell_exit_code() {
    let out = adb().shell("exit 42").await.unwrap();
    assert_eq!(out.exit_code, 42);
}

#[tokio::test]
#[ignore]
async fn shell_stderr() {
    let out = adb().shell("echo err >&2").await.unwrap();
    assert!(out.success());
    assert_eq!(out.stderr_str(), "err");
}

#[tokio::test]
#[ignore]
async fn shell_stream_lines() {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let stream = adb().shell_stream("echo -e 'a\\nb\\nc'").await.unwrap();
    let mut lines = BufReader::new(stream).lines();
    let mut collected = Vec::new();
    while let Some(line) = lines.next_line().await.unwrap() {
        collected.push(line);
    }
    assert_eq!(collected, vec!["a", "b", "c"]);
}

#[tokio::test]
#[ignore]
async fn stat_existing_path() {
    let st = adb().stat("/system/build.prop").await.unwrap();
    assert!(st.exists());
    assert!(st.is_file());
    assert!(st.size > 0);
}

#[tokio::test]
#[ignore]
async fn stat_nonexistent() {
    let st = adb().stat("/nonexistent_path_12345").await.unwrap();
    assert!(!st.exists());
}

#[tokio::test]
#[ignore]
async fn stat_directory() {
    let st = adb().stat("/sdcard").await.unwrap();
    assert!(st.exists());
    assert!(st.is_dir());
}

#[tokio::test]
#[ignore]
async fn push_pull_roundtrip() {
    let adb = adb();
    let data = b"adb-wire integration test data";
    let remote = "/data/local/tmp/adb_wire_test.txt";

    // Push
    let mut reader = std::io::Cursor::new(data.as_slice());
    adb.push(remote, 0o644, 0, &mut reader).await.unwrap();

    // Pull back and compare
    let mut output = Vec::new();
    adb.pull(remote, &mut output).await.unwrap();
    assert_eq!(&output, data);

    // Cleanup
    let _ = adb.shell(&format!("rm -f '{remote}'")).await;
}

#[tokio::test]
#[ignore]
async fn list_dir_data_local_tmp() {
    let adb = adb();
    // Ensure at least one file exists
    let remote = "/data/local/tmp/adb_wire_list_test";
    adb.shell(&format!("touch '{remote}'")).await.unwrap();

    let entries = adb.list_dir("/data/local/tmp").await.unwrap();
    assert!(
        entries.iter().any(|e| e.name == "adb_wire_list_test"),
        "expected file not found in listing"
    );

    // Cleanup
    let _ = adb.shell(&format!("rm -f '{remote}'")).await;
}

#[tokio::test]
#[ignore]
async fn shell_detach_executes() {
    let adb = adb();
    let remote = "/data/local/tmp/adb_wire_detach_test";
    let _ = adb.shell(&format!("rm -f '{remote}'")).await;

    adb.shell_detach(&format!("touch '{remote}'")).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let st = adb.stat(remote).await.unwrap();
    assert!(st.exists(), "shell_detach command did not execute");

    // Cleanup
    let _ = adb.shell(&format!("rm -f '{remote}'")).await;
}

#[tokio::test]
#[ignore]
async fn forward_succeeds() {
    // Verify the protocol exchange works (doesn't test actual data flow).
    adb().forward("tcp:0", "tcp:5555").await.unwrap();
}

#[tokio::test]
#[ignore]
async fn reverse_succeeds() {
    adb().reverse("tcp:0", "tcp:5555").await.unwrap();
}
