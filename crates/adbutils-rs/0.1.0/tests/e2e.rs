//! End-to-end tests against a real device / emulator.
//!
//! Gated behind the `e2e` feature so `cargo test` skips them by default. Run:
//!
//! ```text
//! adb devices                 # confirm exactly one device is connected
//! cargo test --features e2e -- --test-threads=1 --nocapture
//! ```
//!
//! Uses the sole connected device (`any_device`). Set `ANDROID_SERIAL` to target
//! a specific one.
#![cfg(feature = "e2e")]

use adbutils::proto::Network;
use adbutils::AdbClient;

async fn device() -> adbutils::AdbDevice {
    AdbClient::default()
        .any_device()
        .await
        .expect("no device; connect one and authorize USB debugging")
}

#[tokio::test]
async fn server_version_is_sane() {
    let v = AdbClient::default().server_version().await.unwrap();
    assert!(v >= 39, "unexpected adb server version {v}");
}

#[tokio::test]
async fn shell_echo_roundtrip() {
    let d = device().await;
    let out = d.shell("echo hello-adbutils").await.unwrap();
    assert_eq!(out, "hello-adbutils");
}

#[tokio::test]
async fn shell2_reports_exit_code() {
    let d = device().await;
    let ok = d.shell2("true", false).await.unwrap();
    assert_eq!(ok.returncode, 0);
    // Use a subshell: a bare `exit 7` would kill the shell before the v1
    // `; echo X4EXIT:$?` marker runs (inherent to the trick).
    let bad = d.shell2("sh -c 'exit 7'", false).await.unwrap();
    assert_eq!(bad.returncode, 7);
}

#[tokio::test]
async fn getprop_and_features() {
    let d = device().await;
    let model = d.getprop("ro.product.model").await.unwrap();
    assert!(!model.is_empty(), "model should not be empty");
    let feats = d.get_features().await.unwrap();
    assert!(feats.contains("cmd") || feats.contains("shell_v2"), "features: {feats}");
}

#[tokio::test]
async fn window_size_positive() {
    let d = device().await;
    let ws = d.window_size(None).await.unwrap();
    assert!(ws.width > 0 && ws.height > 0, "got {ws:?}");
}

#[tokio::test]
async fn battery_has_level() {
    let d = device().await;
    let b = d.battery().await.unwrap();
    assert!(b.level.unwrap_or(-1) >= 0, "battery: {b:?}");
}

#[tokio::test]
async fn sync_push_pull_roundtrip() {
    let d = device().await;
    let sync = d.sync();
    let payload = b"adbutils-rust sync roundtrip \xf0\x9f\x93\xb1"; // includes UTF-8 emoji bytes
    let remote = "/data/local/tmp/adbutils-rust-e2e.bin";

    let n = sync.push_bytes(payload, remote, 0o644).await.unwrap();
    assert_eq!(n as usize, payload.len());

    let info = sync.stat(remote).await.unwrap();
    assert_eq!(info.size as usize, payload.len());
    assert!(info.mtime.is_some());

    let got = sync.read_bytes(remote).await.unwrap();
    assert_eq!(got, payload);

    d.remove(remote).await.unwrap();
    assert!(!sync.exists(remote).await.unwrap());
}

#[tokio::test]
async fn forward_add_list_remove() {
    let d = device().await;
    let local = "tcp:0"; // let adb pick a port
    // Use an explicit port so we can assert it back.
    let port = d.forward_port("tcp:9008").await.unwrap();
    assert!(port > 0);
    let list = d.forward_list().await.unwrap();
    assert!(list.iter().any(|f| f.remote == "tcp:9008"), "forwards: {list:?}");
    let _ = local;
    d.forward_remove_all().await.unwrap();
}

#[tokio::test]
async fn create_connection_tcp() {
    // Requires a listening service; just assert the API path doesn't panic and
    // errors cleanly when nothing is listening.
    let d = device().await;
    let _ = d.create_connection(Network::Tcp, "1").await; // ok either way
}

#[tokio::test]
#[cfg(feature = "image")]
async fn screenshot_decodes() {
    let d = device().await;
    let img = d.screenshot(None, false).await.unwrap();
    assert!(img.width() > 0 && img.height() > 0);
}

#[tokio::test]
async fn app_current_returns_package() {
    let d = device().await;
    let info = d.app_current().await.unwrap();
    assert!(info.package.contains('.'), "package: {}", info.package);
}

#[tokio::test]
async fn list_packages_nonempty() {
    let d = device().await;
    let pkgs = d.list_packages(&[]).await.unwrap();
    assert!(pkgs.iter().any(|p| p == "android"), "expected 'android' in {} pkgs", pkgs.len());
}
