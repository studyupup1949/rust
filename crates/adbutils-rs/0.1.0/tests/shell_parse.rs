//! Tests for shell-output parsing (battery, window size, packages, app_current)
//! driven through a mock device that returns canned `shell:` output.

use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use adbutils::AdbClient;

const SERIAL: &str = "emulator-5554";

async fn read_command(sock: &mut TcpStream) -> String {
    let mut hdr = [0u8; 4];
    sock.read_exact(&mut hdr).await.unwrap();
    let len = usize::from_str_radix(std::str::from_utf8(&hdr).unwrap(), 16).unwrap();
    let mut buf = vec![0u8; len];
    sock.read_exact(&mut buf).await.unwrap();
    String::from_utf8(buf).unwrap()
}

/// Spawn a device mock where every `shell:<cmd>` is answered by `responder(cmd)`.
/// Handles `host:version` and unlimited transport switches, so multi-shell-call
/// helpers work. Panics if the responder returns `None` for a command.
async fn spawn_shell_device<F>(responder: F) -> AdbClient
where
    F: Fn(&str) -> Option<String> + Send + Sync + 'static,
{
    let responder = Arc::new(responder);
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let (mut sock, _) = listener.accept().await.unwrap();
            let responder = responder.clone();
            tokio::spawn(async move {
                let cmd = read_command(&mut sock).await;
                if cmd == "host:version" {
                    sock.write_all(b"OKAY0004").await.unwrap();
                    sock.write_all(b"0029").await.unwrap();
                    return;
                }
                assert_eq!(cmd, format!("host:tport:serial:{SERIAL}"));
                sock.write_all(b"OKAY").await.unwrap();
                sock.write_all(&[0u8; 8]).await.unwrap();
                let shell_cmd = read_command(&mut sock).await;
                let inner = shell_cmd.strip_prefix("shell:").expect("expected shell: command");
                let out = responder(inner).unwrap_or_else(|| panic!("no canned reply for {inner:?}"));
                sock.write_all(b"OKAY").await.unwrap();
                sock.write_all(out.as_bytes()).await.unwrap();
                let _ = sock.shutdown().await;
            });
        }
    });
    AdbClient::new("127.0.0.1", port, Some(Duration::from_secs(5)))
}

#[tokio::test]
async fn window_size_parses_physical() {
    let adb = spawn_shell_device(|cmd| match cmd {
        "wm size" => Some("Physical size: 1080x1920\n".into()),
        _ => None,
    })
    .await;
    // landscape=Some(false) avoids the extra rotation() shell call.
    let ws = adb.device(SERIAL).window_size(Some(false)).await.unwrap();
    assert_eq!(ws.width, 1080);
    assert_eq!(ws.height, 1920);
}

#[tokio::test]
async fn window_size_prefers_override() {
    let adb = spawn_shell_device(|cmd| match cmd {
        "wm size" => Some("Physical size: 1080x1920\nOverride size: 720x1280\n".into()),
        _ => None,
    })
    .await;
    let ws = adb.device(SERIAL).window_size(Some(false)).await.unwrap();
    assert_eq!((ws.width, ws.height), (720, 1280));
}

#[tokio::test]
async fn list_packages_sorted_and_filtered() {
    let adb = spawn_shell_device(|cmd| {
        if cmd.starts_with("pm list packages") {
            Some("package:com.b.app\npackage:com.a.app\r\n".into())
        } else {
            None
        }
    })
    .await;
    let pkgs = adb.device(SERIAL).list_packages(&["-3"]).await.unwrap();
    assert_eq!(pkgs, vec!["com.a.app".to_string(), "com.b.app".to_string()]);
}

#[tokio::test]
async fn battery_parses_fields() {
    let dump = "Current Battery Service state:\n  \
        AC powered: false\n  USB powered: true\n  Wireless powered: false\n  \
        status: 2\n  health: 2\n  present: true\n  level: 87\n  scale: 100\n  \
        voltage: 4200\n  temperature: 305\n  technology: Li-ion\n";
    let adb = spawn_shell_device(move |cmd| {
        if cmd == "dumpsys battery" {
            Some(dump.to_string())
        } else {
            None
        }
    })
    .await;
    let b = adb.device(SERIAL).battery().await.unwrap();
    assert!(b.usb_powered);
    assert!(!b.ac_powered);
    assert_eq!(b.level, Some(87));
    assert_eq!(b.voltage, Some(4200));
    assert_eq!(b.temperature, Some(30.5));
    assert_eq!(b.technology.as_deref(), Some("Li-ion"));
}

#[tokio::test]
async fn app_current_from_current_focus() {
    let adb = spawn_shell_device(|cmd| match cmd {
        "dumpsys window windows" => Some(
            "  mCurrentFocus=Window{41b37570 u0 com.example.app/com.example.app.MainActivity}\n"
                .into(),
        ),
        _ => None,
    })
    .await;
    let info = adb.device(SERIAL).app_current().await.unwrap();
    assert_eq!(info.package, "com.example.app");
    assert_eq!(info.activity, "com.example.app.MainActivity");
}

#[tokio::test]
async fn getprop_trims() {
    let adb = spawn_shell_device(|cmd| {
        if cmd == "getprop ro.product.model" {
            Some("Pixel 7\n".into())
        } else {
            None
        }
    })
    .await;
    let model = adb.device(SERIAL).getprop("ro.product.model").await.unwrap();
    assert_eq!(model, "Pixel 7");
}
