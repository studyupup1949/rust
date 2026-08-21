//! ADB sync protocol (binary-framed file transfer and stat).

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::{Error, Result};

pub(crate) const SYNC_DATA_MAX: usize = 64 * 1024;

/// Maximum size for error messages from the sync protocol (256 KiB).
const SYNC_MSG_MAX: usize = 256 * 1024;

/// File metadata returned by [`AdbWire::stat`](crate::AdbWire::stat).
///
/// When the device supports STAT2, all fields are populated from the extended
/// response. On older devices only `mode`, `size`, and `mtime` are set (the
/// rest default to zero).
#[derive(Debug, Clone)]
pub struct RemoteStat {
    /// File mode (type + permissions), e.g. `0o100644` for a regular file.
    pub mode: u32,
    /// File size in bytes.
    pub size: u64,
    /// Last modification time as seconds since the Unix epoch.
    pub mtime: u64,
    /// Device ID (STAT2 only).
    pub dev: u64,
    /// Inode number (STAT2 only).
    pub ino: u64,
    /// Number of hard links (STAT2 only).
    pub nlink: u32,
    /// Owner user ID (STAT2 only).
    pub uid: u32,
    /// Owner group ID (STAT2 only).
    pub gid: u32,
    /// Last access time as seconds since the Unix epoch (STAT2 only).
    pub atime: u64,
    /// Status change time as seconds since the Unix epoch (STAT2 only).
    pub ctime: u64,
}

impl RemoteStat {
    /// Returns `true` if the stat result represents an existing file or directory.
    ///
    /// A zeroed-out stat means the path does not exist.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.mode != 0 || self.size != 0 || self.mtime != 0
    }

    fn from_v1(mode: u32, size: u32, mtime: u32) -> Self {
        Self {
            mode,
            size: size as u64,
            mtime: mtime as u64,
            dev: 0, ino: 0, nlink: 0, uid: 0, gid: 0, atime: 0, ctime: 0,
        }
    }

    /// Returns `true` if this is a regular file.
    #[must_use]
    pub fn is_file(&self) -> bool {
        (self.mode & 0o170000) == 0o100000
    }

    /// Returns `true` if this is a directory.
    #[must_use]
    pub fn is_dir(&self) -> bool {
        (self.mode & 0o170000) == 0o040000
    }
}

// -- Low-level sync helpers ---------------------------------------------------

pub(crate) async fn sync_send(
    w: &mut (impl AsyncWrite + Unpin),
    tag: &[u8; 4],
    data: &[u8],
) -> Result<()> {
    let mut pkt = Vec::with_capacity(8 + data.len());
    pkt.extend_from_slice(tag);
    pkt.extend_from_slice(&(data.len() as u32).to_le_bytes());
    pkt.extend_from_slice(data);
    w.write_all(&pkt).await?;
    w.flush().await?;
    Ok(())
}

pub(crate) async fn sync_read_header(
    r: &mut (impl AsyncRead + Unpin),
) -> Result<([u8; 4], u32)> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf).await?;
    let tag: [u8; 4] = buf[..4].try_into().unwrap();
    let len = u32::from_le_bytes(buf[4..8].try_into().unwrap());
    Ok((tag, len))
}

async fn read_sync_fail(r: &mut (impl AsyncRead + Unpin), len: u32) -> Error {
    let msg_len = (len as usize).min(SYNC_MSG_MAX);
    let mut msg = vec![0u8; msg_len];
    match r.read_exact(&mut msg).await {
        Ok(_) => Error::Adb(String::from_utf8_lossy(&msg).to_string()),
        Err(e) => e.into(),
    }
}

// -- STAT ---------------------------------------------------------------------

pub(crate) async fn stat_v1_sync(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    remote_path: &str,
) -> Result<RemoteStat> {
    sync_send(stream, b"STAT", remote_path.as_bytes()).await?;

    let mut resp = [0u8; 16]; // "STAT" + mode(4) + size(4) + mtime(4)
    stream.read_exact(&mut resp).await?;

    if &resp[..4] != b"STAT" {
        return Err(Error::Protocol(format!(
            "expected STAT response, got {:?}",
            String::from_utf8_lossy(&resp[..4])
        )));
    }

    let mode = u32::from_le_bytes(resp[4..8].try_into().unwrap());
    let size = u32::from_le_bytes(resp[8..12].try_into().unwrap());
    let mtime = u32::from_le_bytes(resp[12..16].try_into().unwrap());

    Ok(RemoteStat::from_v1(mode, size, mtime))
}

/// STAT2 response: tag(4) + error(4) + dev(8) + ino(8) + mode(4) + nlink(4) +
///                 uid(4) + gid(4) + size(8) + atime(8) + mtime(8) + ctime(8) = 72 bytes
const STAT2_RESP_LEN: usize = 72;

pub(crate) async fn stat2_sync(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    remote_path: &str,
) -> Result<RemoteStat> {
    sync_send(stream, b"STA2", remote_path.as_bytes()).await?;

    let mut resp = [0u8; STAT2_RESP_LEN];
    stream.read_exact(&mut resp).await?;

    let tag = &resp[..4];
    if tag != b"STA2" {
        return Err(Error::Protocol(format!(
            "expected STA2 response, got {:?}",
            String::from_utf8_lossy(tag)
        )));
    }

    let error = u32::from_le_bytes(resp[4..8].try_into().unwrap());
    if error != 0 {
        return Err(Error::Adb(format!("STA2 error: {error}")));
    }

    let dev   = u64::from_le_bytes(resp[8..16].try_into().unwrap());
    let ino   = u64::from_le_bytes(resp[16..24].try_into().unwrap());
    let mode  = u32::from_le_bytes(resp[24..28].try_into().unwrap());
    let nlink = u32::from_le_bytes(resp[28..32].try_into().unwrap());
    let uid   = u32::from_le_bytes(resp[32..36].try_into().unwrap());
    let gid   = u32::from_le_bytes(resp[36..40].try_into().unwrap());
    let size  = u64::from_le_bytes(resp[40..48].try_into().unwrap());
    let atime = u64::from_le_bytes(resp[48..56].try_into().unwrap());
    let mtime = u64::from_le_bytes(resp[56..64].try_into().unwrap());
    let ctime = u64::from_le_bytes(resp[64..72].try_into().unwrap());

    Ok(RemoteStat { mode, size, mtime, dev, ino, nlink, uid, gid, atime, ctime })
}

// -- LIST (directory listing) --------------------------------------------------

/// Entry from a remote directory listing.
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// File name (without path).
    pub name: String,
    /// File mode (type + permissions).
    pub mode: u32,
    /// File size in bytes (32-bit, from sync v1 protocol).
    pub size: u32,
    /// Last modification time as seconds since the Unix epoch.
    pub mtime: u32,
}

pub(crate) async fn list_sync(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    remote_path: &str,
) -> Result<Vec<DirEntry>> {
    sync_send(stream, b"LIST", remote_path.as_bytes()).await?;

    let mut entries = Vec::new();
    loop {
        // DENT header: tag(4) + mode(4) + size(4) + mtime(4) + namelen(4) = 20 bytes
        // DONE uses the same 20-byte layout.
        let mut header = [0u8; 20];
        stream.read_exact(&mut header).await?;

        let tag: [u8; 4] = header[..4].try_into().unwrap();
        match &tag {
            b"DENT" => {
                let mode = u32::from_le_bytes(header[4..8].try_into().unwrap());
                let size = u32::from_le_bytes(header[8..12].try_into().unwrap());
                let mtime = u32::from_le_bytes(header[12..16].try_into().unwrap());
                let name_len = u32::from_le_bytes(header[16..20].try_into().unwrap()) as usize;

                if name_len > SYNC_DATA_MAX {
                    return Err(Error::Protocol("DENT name too long".into()));
                }
                let mut name_buf = vec![0u8; name_len];
                if name_len > 0 {
                    stream.read_exact(&mut name_buf).await?;
                }
                let name = String::from_utf8_lossy(&name_buf).to_string();

                if name != "." && name != ".." {
                    entries.push(DirEntry { name, mode, size, mtime });
                }
            }
            b"DONE" => break,
            b"FAIL" => {
                let msg_len = u32::from_le_bytes(header[4..8].try_into().unwrap());
                return Err(read_sync_fail(stream, msg_len).await);
            }
            _ => {
                return Err(Error::Protocol(format!(
                    "unexpected LIST tag: {:?}",
                    String::from_utf8_lossy(&tag)
                )));
            }
        }
    }

    Ok(entries)
}

// -- RECV (pull) --------------------------------------------------------------

pub(crate) async fn pull_sync(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    remote_path: &str,
    writer: &mut (impl AsyncWrite + Unpin),
) -> Result<u64> {
    sync_send(stream, b"RECV", remote_path.as_bytes()).await?;

    let mut total: u64 = 0;
    let mut buf = vec![0u8; SYNC_DATA_MAX];
    loop {
        let (tag, len) = sync_read_header(stream).await?;
        match &tag {
            b"DATA" => {
                let mut remaining = len as usize;
                while remaining > 0 {
                    let n = remaining.min(buf.len());
                    stream.read_exact(&mut buf[..n]).await?;
                    writer.write_all(&buf[..n]).await?;
                    remaining -= n;
                    total += n as u64;
                }
            }
            b"DONE" => break,
            b"FAIL" => return Err(read_sync_fail(stream, len).await),
            _ => {
                return Err(Error::Protocol(format!(
                    "unexpected sync tag: {:?}",
                    String::from_utf8_lossy(&tag)
                )))
            }
        }
    }

    sync_send(stream, b"QUIT", &[]).await?;
    Ok(total)
}

// -- SEND (push) --------------------------------------------------------------

pub(crate) async fn push_sync(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
    remote_path: &str,
    mode: u32,
    mtime: u32,
    reader: &mut (impl AsyncRead + Unpin),
) -> Result<()> {
    let header = format!("{remote_path},{mode}");
    sync_send(stream, b"SEND", header.as_bytes()).await?;

    let mut buf = vec![0u8; 8 + SYNC_DATA_MAX];
    loop {
        let n = read_fill(reader, &mut buf[8..]).await?;
        if n == 0 {
            break;
        }
        buf[..4].copy_from_slice(b"DATA");
        buf[4..8].copy_from_slice(&(n as u32).to_le_bytes());
        stream.write_all(&buf[..8 + n]).await?;
    }

    let mut done = [0u8; 8];
    done[..4].copy_from_slice(b"DONE");
    done[4..8].copy_from_slice(&mtime.to_le_bytes());
    stream.write_all(&done).await?;
    stream.flush().await?;

    let (tag, len) = sync_read_header(stream).await?;
    match &tag {
        b"OKAY" => Ok(()),
        b"FAIL" => Err(read_sync_fail(stream, len).await),
        _ => Err(Error::Protocol(format!(
            "unexpected sync response: {:?}",
            String::from_utf8_lossy(&tag)
        ))),
    }
}

/// Read until `buf` is full or EOF. Returns bytes read.
async fn read_fill(r: &mut (impl AsyncRead + Unpin), buf: &mut [u8]) -> std::io::Result<usize> {
    let mut pos = 0;
    while pos < buf.len() {
        match r.read(&mut buf[pos..]).await? {
            0 => break,
            n => pos += n,
        }
    }
    Ok(pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Mock stream with separate read/write buffers.
    struct MockStream {
        read: Cursor<Vec<u8>>,
        written: Vec<u8>,
    }

    impl MockStream {
        fn from_response(data: Vec<u8>) -> Self {
            Self {
                read: Cursor::new(data),
                written: Vec::new(),
            }
        }
    }

    impl AsyncRead for MockStream {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let pos = self.read.position() as usize;
            let inner = self.read.get_ref();
            let remaining = &inner[pos..];
            let n = remaining.len().min(buf.remaining());
            buf.put_slice(&remaining[..n]);
            self.read.set_position((pos + n) as u64);
            std::task::Poll::Ready(Ok(()))
        }
    }

    impl AsyncWrite for MockStream {
        fn poll_write(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            self.written.extend_from_slice(buf);
            std::task::Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn sync_send_roundtrip() {
        let mut buf = Vec::new();
        sync_send(&mut buf, b"RECV", b"/sdcard/test.txt").await.unwrap();

        assert_eq!(&buf[..4], b"RECV");
        let len = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        assert_eq!(len, 16);
        assert_eq!(&buf[8..], b"/sdcard/test.txt");
    }

    #[tokio::test]
    async fn sync_header_roundtrip() {
        let bytes: Vec<u8> = b"DATA"
            .iter()
            .chain(&100u32.to_le_bytes())
            .copied()
            .collect();
        let mut cur = Cursor::new(bytes);
        let (tag, len) = sync_read_header(&mut cur).await.unwrap();
        assert_eq!(&tag, b"DATA");
        assert_eq!(len, 100);
    }

    #[tokio::test]
    async fn pull_sync_single_chunk() {
        let payload = b"hello world";
        let mut wire = Vec::new();
        wire.extend_from_slice(b"DATA");
        wire.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        wire.extend_from_slice(payload);
        wire.extend_from_slice(b"DONE");
        wire.extend_from_slice(&0u32.to_le_bytes());

        let mut stream = MockStream::from_response(wire);
        let mut output = Vec::new();
        let n = pull_sync(&mut stream, "/sdcard/test.txt", &mut output).await.unwrap();

        assert_eq!(n, 11);
        assert_eq!(&output, b"hello world");
        assert_eq!(&stream.written[..4], b"RECV");
    }

    #[tokio::test]
    async fn pull_sync_multiple_chunks() {
        let mut wire = Vec::new();
        for chunk in [b"aaa".as_slice(), b"bbb"] {
            wire.extend_from_slice(b"DATA");
            wire.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
            wire.extend_from_slice(chunk);
        }
        wire.extend_from_slice(b"DONE");
        wire.extend_from_slice(&0u32.to_le_bytes());

        let mut stream = MockStream::from_response(wire);
        let mut output = Vec::new();
        let n = pull_sync(&mut stream, "/test", &mut output).await.unwrap();

        assert_eq!(n, 6);
        assert_eq!(&output, b"aaabbb");
    }

    #[tokio::test]
    async fn pull_sync_fail() {
        let msg = b"file not found";
        let mut wire = Vec::new();
        wire.extend_from_slice(b"FAIL");
        wire.extend_from_slice(&(msg.len() as u32).to_le_bytes());
        wire.extend_from_slice(msg);

        let mut stream = MockStream::from_response(wire);
        let mut output = Vec::new();
        let err = pull_sync(&mut stream, "/nope", &mut output).await.unwrap_err();
        assert!(matches!(err, Error::Adb(m) if m == "file not found"));
    }

    #[tokio::test]
    async fn push_sync_roundtrip() {
        let response: Vec<u8> = b"OKAY"
            .iter()
            .chain(&0u32.to_le_bytes())
            .copied()
            .collect();
        let mut stream = MockStream::from_response(response);

        let data = b"file contents";
        let mut reader = Cursor::new(data.as_slice());
        push_sync(&mut stream, "/sdcard/out.txt", 0o644, 1000, &mut reader).await.unwrap();

        assert_eq!(&stream.written[..4], b"SEND");
        let w = &stream.written;
        let tail = &w[w.len() - 8..];
        assert_eq!(&tail[..4], b"DONE");
        assert_eq!(u32::from_le_bytes(tail[4..8].try_into().unwrap()), 1000);
    }

    #[tokio::test]
    async fn read_fill_partial() {
        let data = b"hello";
        let mut reader = Cursor::new(data.as_slice());
        let mut buf = [0u8; 10];
        let n = read_fill(&mut reader, &mut buf).await.unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf[..5], b"hello");
    }

    // -- stat v1 tests --------------------------------------------------------

    #[tokio::test]
    async fn stat_v1_existing_file() {
        let mut wire = Vec::new();
        wire.extend_from_slice(b"STAT");
        wire.extend_from_slice(&0o100644u32.to_le_bytes());
        wire.extend_from_slice(&1024u32.to_le_bytes());
        wire.extend_from_slice(&1700000000u32.to_le_bytes());

        let mut stream = MockStream::from_response(wire);
        let st = stat_v1_sync(&mut stream, "/sdcard/test.txt").await.unwrap();

        assert!(st.exists());
        assert!(st.is_file());
        assert!(!st.is_dir());
        assert_eq!(st.size, 1024);
        assert_eq!(st.mtime, 1700000000);
    }

    #[tokio::test]
    async fn stat_v1_directory() {
        let mut wire = Vec::new();
        wire.extend_from_slice(b"STAT");
        wire.extend_from_slice(&0o040755u32.to_le_bytes());
        wire.extend_from_slice(&4096u32.to_le_bytes());
        wire.extend_from_slice(&1700000000u32.to_le_bytes());

        let mut stream = MockStream::from_response(wire);
        let st = stat_v1_sync(&mut stream, "/sdcard").await.unwrap();

        assert!(st.exists());
        assert!(!st.is_file());
        assert!(st.is_dir());
    }

    #[tokio::test]
    async fn stat_v1_nonexistent() {
        let mut wire = Vec::new();
        wire.extend_from_slice(b"STAT");
        wire.extend_from_slice(&0u32.to_le_bytes());
        wire.extend_from_slice(&0u32.to_le_bytes());
        wire.extend_from_slice(&0u32.to_le_bytes());

        let mut stream = MockStream::from_response(wire);
        let st = stat_v1_sync(&mut stream, "/nonexistent").await.unwrap();

        assert!(!st.exists());
    }

    // -- stat2 tests ----------------------------------------------------------

    #[tokio::test]
    async fn stat2_existing_file() {
        let mut wire = Vec::new();
        wire.extend_from_slice(b"STA2");
        wire.extend_from_slice(&0u32.to_le_bytes());         // error
        wire.extend_from_slice(&1u64.to_le_bytes());          // dev
        wire.extend_from_slice(&12345u64.to_le_bytes());      // ino
        wire.extend_from_slice(&0o100644u32.to_le_bytes());   // mode
        wire.extend_from_slice(&1u32.to_le_bytes());          // nlink
        wire.extend_from_slice(&1000u32.to_le_bytes());       // uid
        wire.extend_from_slice(&1000u32.to_le_bytes());       // gid
        wire.extend_from_slice(&5_000_000_000u64.to_le_bytes()); // size (>4GB)
        wire.extend_from_slice(&1700000000u64.to_le_bytes()); // atime
        wire.extend_from_slice(&1700000001u64.to_le_bytes()); // mtime
        wire.extend_from_slice(&1700000002u64.to_le_bytes()); // ctime

        let mut stream = MockStream::from_response(wire);
        let st = stat2_sync(&mut stream, "/sdcard/large.bin").await.unwrap();

        assert!(st.exists());
        assert!(st.is_file());
        assert_eq!(st.size, 5_000_000_000);
        assert_eq!(st.mtime, 1700000001);
        assert_eq!(st.uid, 1000);
        assert_eq!(st.ino, 12345);
    }

    #[tokio::test]
    async fn stat2_error_returns_err() {
        let mut wire = Vec::new();
        wire.extend_from_slice(b"STA2");
        wire.extend_from_slice(&1u32.to_le_bytes()); // error != 0
        wire.extend_from_slice(&[0u8; 64]);           // remaining fields

        let mut stream = MockStream::from_response(wire);
        assert!(stat2_sync(&mut stream, "/nope").await.is_err());
    }

    // -- list tests -----------------------------------------------------------

    fn make_dent(name: &str, mode: u32, size: u32, mtime: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"DENT");
        buf.extend_from_slice(&mode.to_le_bytes());
        buf.extend_from_slice(&size.to_le_bytes());
        buf.extend_from_slice(&mtime.to_le_bytes());
        buf.extend_from_slice(&(name.len() as u32).to_le_bytes());
        buf.extend_from_slice(name.as_bytes());
        buf
    }

    fn make_done() -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"DONE");
        buf.extend_from_slice(&[0u8; 16]); // mode + size + mtime + namelen = 0
        buf
    }

    #[tokio::test]
    async fn list_sync_entries() {
        let mut wire = Vec::new();
        wire.extend(make_dent(".", 0o040755, 0, 0));
        wire.extend(make_dent("..", 0o040755, 0, 0));
        wire.extend(make_dent("hello.txt", 0o100644, 42, 1700000000));
        wire.extend(make_dent("subdir", 0o040755, 4096, 1700000000));
        wire.extend(make_done());

        let mut stream = MockStream::from_response(wire);
        let entries = list_sync(&mut stream, "/sdcard").await.unwrap();

        assert_eq!(entries.len(), 2); // . and .. filtered
        assert_eq!(entries[0].name, "hello.txt");
        assert_eq!(entries[0].size, 42);
        assert_eq!(entries[1].name, "subdir");
    }

    #[tokio::test]
    async fn list_sync_empty_dir() {
        let wire = make_done();
        let mut stream = MockStream::from_response(wire);
        let entries = list_sync(&mut stream, "/empty").await.unwrap();
        assert!(entries.is_empty());
    }
}
