//! Unit tests against a mock adb server that speaks the smartsocket protocol.
//! No device required. Validates M1 host-command framing byte-exact.

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use adbutils::AdbClient;

/// Read one smartsocket command (4 hex length + payload) from the client.
async fn read_command(sock: &mut TcpStream) -> String {
    let mut hdr = [0u8; 4];
    sock.read_exact(&mut hdr).await.unwrap();
    let len = usize::from_str_radix(std::str::from_utf8(&hdr).unwrap(), 16).unwrap();
    let mut buf = vec![0u8; len];
    sock.read_exact(&mut buf).await.unwrap();
    String::from_utf8(buf).unwrap()
}

/// Write OKAY + a length-prefixed reply block.
async fn write_okay_block(sock: &mut TcpStream, payload: &str) {
    sock.write_all(b"OKAY").await.unwrap();
    let header = format!("{:04x}", payload.len());
    sock.write_all(header.as_bytes()).await.unwrap();
    sock.write_all(payload.as_bytes()).await.unwrap();
}

/// Spawn a mock server that handles a fixed sequence of (expected_cmd → reply)
/// exchanges, one per accepted connection. Returns the bound port.
async fn spawn_mock(exchanges: Vec<(&'static str, MockReply)>) -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        for (expected, reply) in exchanges {
            let (mut sock, _) = listener.accept().await.unwrap();
            let cmd = read_command(&mut sock).await;
            assert_eq!(cmd, expected, "unexpected command");
            match reply {
                MockReply::OkayBlock(s) => write_okay_block(&mut sock, s).await,
                MockReply::Fail(s) => {
                    sock.write_all(b"FAIL").await.unwrap();
                    let header = format!("{:04x}", s.len());
                    sock.write_all(header.as_bytes()).await.unwrap();
                    sock.write_all(s.as_bytes()).await.unwrap();
                }
                MockReply::OkayThenClose(s) => {
                    // used for shell: OKAY then raw bytes then EOF
                    sock.write_all(b"OKAY").await.unwrap();
                    sock.write_all(s.as_bytes()).await.unwrap();
                    let _ = sock.shutdown().await;
                }
            }
        }
    });
    port
}

enum MockReply {
    OkayBlock(&'static str),
    Fail(&'static str),
    #[allow(dead_code)]
    OkayThenClose(&'static str),
}

fn client(port: u16) -> AdbClient {
    AdbClient::new("127.0.0.1", port, Some(Duration::from_secs(5)))
}

#[tokio::test]
async fn server_version_parses_hex() {
    let port = spawn_mock(vec![("host:version", MockReply::OkayBlock("0029"))]).await;
    let v = client(port).server_version().await.unwrap();
    assert_eq!(v, 0x29); // 41
}

#[tokio::test]
async fn device_list_filters_state_device() {
    let payload = "emulator-5554\tdevice\nabcd1234\toffline\n";
    let port = spawn_mock(vec![("host:devices", MockReply::OkayBlock(payload))]).await;
    let devices = client(port).device_list().await.unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].serial(), Some("emulator-5554"));
}

#[tokio::test]
async fn list_extended_parses_tags() {
    let payload = "emulator-5554         device product:sdk model:Android transport_id:1\n";
    let port = spawn_mock(vec![("host:devices-l", MockReply::OkayBlock(payload))]).await;
    let infos = client(port).list(true).await.unwrap();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].transport_id(), Some(1));
    assert_eq!(infos[0].tags.get("model").map(String::as_str), Some("Android"));
}

#[tokio::test]
async fn fail_reply_surfaces_error() {
    let port = spawn_mock(vec![(
        "host:connect:1.2.3.4:5555",
        MockReply::Fail("no such host"),
    )])
    .await;
    let err = client(port).connect("1.2.3.4:5555", None).await.unwrap_err();
    assert!(err.to_string().contains("no such host"), "got: {err}");
}

#[tokio::test]
async fn shell_via_tport_discards_transport_id() {
    // device.shell flow (adb server ≥ 41):
    //   conn 1: host:version → OKAY + "0029"
    //   conn 2: host:tport:serial:<s> → OKAY + 8-byte tid,
    //           then shell:echo hi → OKAY + "hi" + EOF
    // The client opens the transport socket before the version socket but only
    // writes to it after server_version returns, so connections must be handled
    // concurrently. Dispatch each accepted connection by its first command.
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        for _ in 0..2 {
            let (mut sock, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let cmd = read_command(&mut sock).await;
                if cmd == "host:version" {
                    write_okay_block(&mut sock, "0029").await;
                } else {
                    assert_eq!(cmd, "host:tport:serial:emulator-5554");
                    sock.write_all(b"OKAY").await.unwrap();
                    sock.write_all(&[0u8; 8]).await.unwrap(); // 8-byte transport id
                    assert_eq!(read_command(&mut sock).await, "shell:echo hi");
                    sock.write_all(b"OKAY").await.unwrap();
                    sock.write_all(b"hi\n").await.unwrap();
                    let _ = sock.shutdown().await;
                }
            });
        }
    });

    let out = client(port).device("emulator-5554").shell("echo hi").await.unwrap();
    assert_eq!(out, "hi"); // rstrip removes the trailing newline
}
