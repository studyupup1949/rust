//! Mock-server tests for device-context protocols (sync, shell v2, reverse).
//! Each device op first opens a `host:version` connection, then a
//! `host:tport:serial:<s>` transport switch (OKAY + 8-byte tid) before the
//! device-context exchange. The harness handles the first two automatically and
//! dispatches the device-context connection to a per-test handler.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

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

/// Spawn a server that answers `host:version` and one transport-switch, then
/// runs `handler` on the device-context socket (already past the tport OKAY+tid).
async fn spawn_device<F, Fut>(handler: F) -> AdbClient
where
    F: FnOnce(TcpStream) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send,
{
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    // The client opens the transport socket before the version socket, so
    // connections must be served concurrently. The handler is FnOnce; the one
    // transport connection takes it.
    let handler = Arc::new(Mutex::new(Some(handler)));
    tokio::spawn(async move {
        loop {
            let (mut sock, _) = listener.accept().await.unwrap();
            let handler = handler.clone();
            tokio::spawn(async move {
                let cmd = read_command(&mut sock).await;
                if cmd == "host:version" {
                    sock.write_all(b"OKAY0004").await.unwrap();
                    sock.write_all(b"0029").await.unwrap();
                } else {
                    assert_eq!(cmd, format!("host:tport:serial:{SERIAL}"));
                    sock.write_all(b"OKAY").await.unwrap();
                    sock.write_all(&[0u8; 8]).await.unwrap();
                    let h = handler.lock().await.take();
                    if let Some(h) = h {
                        h(sock).await;
                    }
                }
            });
        }
    });
    AdbClient::new("127.0.0.1", port, Some(Duration::from_secs(5)))
}

#[tokio::test]
async fn sync_stat_parses_mode_size_mtime() {
    let adb = spawn_device(|mut sock| async move {
        assert_eq!(read_command(&mut sock).await, "sync:");
        sock.write_all(b"OKAY").await.unwrap(); // sync session established
        // read STAT + u32-le len + path
        let mut head = [0u8; 8];
        sock.read_exact(&mut head).await.unwrap();
        assert_eq!(&head[0..4], b"STAT");
        let plen = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
        let mut path = vec![0u8; plen];
        sock.read_exact(&mut path).await.unwrap();
        assert_eq!(&path, b"/data/local/tmp");
        // reply STAT + mode,size,mtime (u32-le)
        sock.write_all(b"STAT").await.unwrap();
        sock.write_all(&0o040755u32.to_le_bytes()).await.unwrap();
        sock.write_all(&4096u32.to_le_bytes()).await.unwrap();
        sock.write_all(&1_600_000_000u32.to_le_bytes()).await.unwrap();
    })
    .await;

    let info = adb.device(SERIAL).sync().stat("/data/local/tmp").await.unwrap();
    assert_eq!(info.mode, 0o040755);
    assert_eq!(info.size, 4096);
    assert!(info.mtime.is_some());
}

#[tokio::test]
async fn sync_push_sends_send_data_done_and_reads_okay() {
    let adb = spawn_device(|mut sock| async move {
        assert_eq!(read_command(&mut sock).await, "sync:");
        sock.write_all(b"OKAY").await.unwrap(); // sync session established
        // SEND + len + "<dst>,<mode>"
        let mut head = [0u8; 8];
        sock.read_exact(&mut head).await.unwrap();
        assert_eq!(&head[0..4], b"SEND");
        let plen = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
        let mut path = vec![0u8; plen];
        sock.read_exact(&mut path).await.unwrap();
        let path = String::from_utf8(path).unwrap();
        assert_eq!(path, format!("/data/local/tmp/x,{}", 0o100000 | 0o644));
        // DATA frame
        let mut dh = [0u8; 8];
        sock.read_exact(&mut dh).await.unwrap();
        assert_eq!(&dh[0..4], b"DATA");
        let dlen = u32::from_le_bytes([dh[4], dh[5], dh[6], dh[7]]) as usize;
        let mut payload = vec![0u8; dlen];
        sock.read_exact(&mut payload).await.unwrap();
        assert_eq!(&payload, b"hello");
        // DONE + mtime
        let mut done = [0u8; 8];
        sock.read_exact(&mut done).await.unwrap();
        assert_eq!(&done[0..4], b"DONE");
        // reply OKAY + 4 bytes (ignored length)
        sock.write_all(b"OKAY\0\0\0\0").await.unwrap();
    })
    .await;

    let n = adb
        .device(SERIAL)
        .sync()
        .push_bytes(b"hello", "/data/local/tmp/x", 0o644)
        .await
        .unwrap();
    assert_eq!(n, 5);
}

#[tokio::test]
async fn sync_recv_reads_data_until_done() {
    let adb = spawn_device(|mut sock| async move {
        assert_eq!(read_command(&mut sock).await, "sync:");
        sock.write_all(b"OKAY").await.unwrap(); // sync session established
        let mut head = [0u8; 8];
        sock.read_exact(&mut head).await.unwrap();
        assert_eq!(&head[0..4], b"RECV");
        let plen = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
        let mut path = vec![0u8; plen];
        sock.read_exact(&mut path).await.unwrap();
        // DATA "abc", DATA "de", DONE
        sock.write_all(b"DATA").await.unwrap();
        sock.write_all(&3u32.to_le_bytes()).await.unwrap();
        sock.write_all(b"abc").await.unwrap();
        sock.write_all(b"DATA").await.unwrap();
        sock.write_all(&2u32.to_le_bytes()).await.unwrap();
        sock.write_all(b"de").await.unwrap();
        sock.write_all(b"DONE").await.unwrap();
        sock.write_all(&0u32.to_le_bytes()).await.unwrap();
    })
    .await;

    let data = adb.device(SERIAL).sync().read_bytes("/x").await.unwrap();
    assert_eq!(&data, b"abcde");
}

#[tokio::test]
async fn shell_v2_frames_stdout_and_exit() {
    let adb = spawn_device(|mut sock| async move {
        assert_eq!(read_command(&mut sock).await, "shell,v2:echo hi");
        sock.write_all(b"OKAY").await.unwrap();
        // stdout frame: id=1, len=3, "hi\n"
        sock.write_all(&[1u8]).await.unwrap();
        sock.write_all(&3u32.to_le_bytes()).await.unwrap();
        sock.write_all(b"hi\n").await.unwrap();
        // exit frame: id=3, len=1, code=0
        sock.write_all(&[3u8]).await.unwrap();
        sock.write_all(&1u32.to_le_bytes()).await.unwrap();
        sock.write_all(&[0u8]).await.unwrap();
    })
    .await;

    // Drive the low-level v2 entry point directly (shell2 would first fetch
    // `features`, which this single-exchange mock doesn't serve).
    let raw = adb.device(SERIAL).shell_v2("echo hi").await.unwrap();
    assert_eq!(raw.returncode, 0);
    assert_eq!(String::from_utf8_lossy(&raw.stdout), "hi\n");
}
